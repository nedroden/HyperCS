//! Password hashing and session token helpers. No HTTP types here.
//!
//! Argon2 is CPU- and memory-hard on purpose; the `*_async` variants run it
//! on the blocking pool so request handling on the async workers is not
//! stalled.

use std::sync::OnceLock;

use argon2::password_hash::{SaltString, rand_core::OsRng as PhcOsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Argon2id (the `argon2` crate default) with a random salt; PHC string out.
pub fn hash_password(password: &str) -> Option<String> {
    let salt = SaltString::generate(&mut PhcOsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .ok()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub async fn hash_password_async(password: String) -> Option<String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .ok()
        .flatten()
}

pub async fn verify_password_async(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .unwrap_or(false)
}

/// A valid Argon2 hash of a throwaway password, computed once. Verifying
/// against it when the username does not exist keeps login timing the same
/// for existing and unknown usernames.
fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| hash_password("hypercs-dummy-password").unwrap_or_default())
}

pub async fn verify_dummy_async(password: String) {
    let _ = verify_password_async(password, dummy_hash().to_string()).await;
}

/// A random, URL-safe opaque token (used for both session tokens and CSRF tokens).
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Only the hash of a session token is ever stored.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// Constant-time string comparison (CSRF tokens).
pub fn tokens_equal(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("correct horse battery staple", &hash));
        assert!(!verify_password("wrong password", &hash));
    }

    #[test]
    fn tokens_are_unique_and_hash_deterministically() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
        assert_eq!(hash_token(&a), hash_token(&a));
        assert_ne!(hash_token(&a), hash_token(&b));
    }

    #[test]
    fn constant_time_compare() {
        assert!(tokens_equal("abc", "abc"));
        assert!(!tokens_equal("abc", "abd"));
        assert!(!tokens_equal("abc", "abcd"));
        assert!(!tokens_equal("", "x"));
    }

    #[test]
    fn dummy_hash_is_a_valid_hash() {
        assert!(PasswordHash::new(dummy_hash()).is_ok());
    }
}
