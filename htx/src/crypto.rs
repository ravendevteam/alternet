//! Cryptographic primitives for the Betanet HTX protocol.
//!
//! This module provides a high-level API for the cryptographic operations
//! required by the specification, wrapping well-vetted Rust crypto libraries.

use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit, Nonce};
use ed25519_dalek::{Keypair, PublicKey as EdPublicKey, Signature, Signer};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

// Constants from the specification
pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;
pub const SIGNATURE_SIZE: usize = 64;

/// A symmetric encryption key.
#[derive(Clone)]
pub struct SymmetricKey([u8; KEY_SIZE]);

/// A Diffie-Hellman secret key.
pub struct DhSecretKey(StaticSecret);

/// A Diffie-Hellman public key.
pub struct DhPublicKey(PublicKey);

/// An Ed25519 keypair for signing.
pub struct SignKeypair(Keypair);

/// An Ed25519 verifying key (public key).
pub struct SignPublicKey(EdPublicKey);


// TODO: Implement wrapper functions for:
// - AEAD encryption/decryption (ChaCha20-Poly1305)
// - HKDF key derivation
// - Ed25519 signing and verification
// - X25519 key exchange
