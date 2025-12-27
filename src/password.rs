use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Salt};
use tracing::error;

use crate::error::HoofprintError;

const PASSWORD_SALT: &str = "ThisIsInsecureButBetterThanNothing12345";

/// Hash a password using Argon2 algorithm ready for storage
pub(crate) fn hash_password(password: &str) -> Result<String, HoofprintError> {
    let argon2 = Argon2::default();
    let base64_salt = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD_NO_PAD,
        PASSWORD_SALT,
    );
    let salt: Salt = base64_salt.as_str().try_into()?;

    argon2
        .hash_password(password.as_bytes(), salt)
        .map(|hash| hash.to_string())
        .map_err(|err| {
            error!("Failed to hash password: {err}");
            HoofprintError::from(err)
        })
}

/// Verify an input password against a stored hashed password
pub(crate) fn verify_password(input_password: &str, db_hashed: &str) -> Result<(), HoofprintError> {
    let parsed_hash = argon2::PasswordHash::new(db_hashed)?;
    Argon2::default()
        .verify_password(input_password.as_bytes(), &parsed_hash)
        .map_err(|_| HoofprintError::Authentication)
}

#[test]
fn test_password_hashing() {
    let password = crate::get_random_password(16);
    let hashed = hash_password(&password).expect("Hashing failed");
    dbg!(&password, &hashed);
    assert!(verify_password(&password, &hashed).is_ok());
    assert!(verify_password("WrongPassword", &hashed).is_err());
    assert!(hashed.contains("argon2id"))
}
