// -*- mode: rust; -*-
//
// This file is part of schnorrkel.
// Copyright (c) 2019 Web 3 Foundation
// See LICENSE for licensing information.
//
// Authors:
// - Jeff Burdges <jeff@web3.foundation>

//! Encryption using schnorrkel keys


// use rand_core::{RngCore,CryptoRng};

#[cfg(feature = "aead")]
use ::aead::{NewAead, generic_array::{GenericArray}};

#[cfg(feature = "aead")]
use curve25519_dalek::{
    ristretto::{CompressedRistretto}, // RistrettoPoint
    // scalar::Scalar,
};

use super::{SecretKey,PublicKey,Keypair,SignatureResult};
use crate::context::SigningTranscript;

#[cfg(feature = "aead")]
use crate::cert::ECQVCertPublic;


impl SecretKey {
    /// Commit the results of a key exchange into a transcript
    pub fn commit_key_exchange<T>(&self, t: &mut T, ctx: &'static [u8], public: &PublicKey) 
    where T: SigningTranscript
    {
        let p = &self.key * public.as_point();
        t.commit_point(ctx,& p.compress());
    }

    /// An AEAD from a key exchange with the specified public key.
    #[cfg(feature = "aead")]
    pub fn aead_without_cert<AEAD: NewAead>(&self, ctx: &[u8], public: &PublicKey) -> AEAD {
        let mut t = merlin::Transcript::new(b"KEX");
        t.append_message(b"ctx",ctx);
        self.commit_key_exchange(&mut t,b"kex",public);
        let mut key: GenericArray<u8, <AEAD as NewAead>::KeySize> = Default::default();
        t.challenge_bytes(b"",key.as_mut_slice());
        AEAD::new(key)
    }

    /// Reciever's AEAD with ECQV certificate.
    ///
    /// Returns the AEAD constructed from an ephemeral key exchange
    /// with the public key computed form the sender's public key
    /// and their implicit ECQV certificate.
    #[cfg(feature = "aead")]
    pub fn reciever_aead_with_ecqv_cert<T,AEAD>(
        &self, 
        t: T, 
        cert_public: &ECQVCertPublic, 
        public: &PublicKey,
    ) -> SignatureResult<AEAD> 
    where T: SigningTranscript, AEAD: NewAead
    {
        let epk = public.open_ecqv_cert(t,cert_public) ?;
        Ok(self.aead_without_cert(b"",&epk))
    }
}

impl PublicKey {
    /// Initalize an AEAD from an ephemeral key exchange with the public key `self`.
    ///
    /// Returns the ephemeral public key and AEAD.
    #[cfg(feature = "aead")]
    pub fn init_aead_without_cert<AEAD: NewAead>(&self, ctx: &[u8]) -> (CompressedRistretto,AEAD) 
    {
        let secret = SecretKey::generate();
        let aead = secret.aead_without_cert(ctx,self);
        (secret.to_public().into_compressed(), aead)
    }
}

impl Keypair {
    /// Sender's AEAD with ECQV certificate.
    ///
    /// Along with the AEAD, we return the implicit ECQV certificate
    /// from which the reciever recreates the ephemeral public key.
    #[cfg(feature = "aead")]
    pub fn sender_aead_with_ecqv_cert<T,AEAD>(&self, t: T, public: &PublicKey) -> (ECQVCertPublic,AEAD) 
    where T: SigningTranscript+Clone, AEAD: NewAead
    {
        let (cert,secret) = self.issue_self_ecqv_cert(t);
        let aead = secret.aead_without_cert(b"",&public);
        (cert, aead)
    }
}
