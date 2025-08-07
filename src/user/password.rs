use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use error_stack::Report;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Password Error: {0}")]
pub struct PasswordError(pub String);

pub enum PasswordState {
    Valid,
    ValidRehashed(Password),
    Invalid,
}

impl PasswordState {
    pub fn is_valid(&self) -> bool {
        match self {
            PasswordState::Valid => true,
            PasswordState::ValidRehashed(_) => true,
            PasswordState::Invalid => false,
        }
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    pub fn is_valid_rehashed(&self) -> bool {
        match self {
            PasswordState::ValidRehashed(_) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
pub enum Password {
    /// Argon2id
    Version1 { argon2: String },
}

impl Password {
    pub fn hash_password(password: String) -> Result<Self, Report<PasswordError>> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| PasswordError("Failed to hash password".to_string()))?
            .to_string();

        Ok(Password::Version1 {
            argon2: password_hash,
        })
    }

    pub fn verify_password(
        password_hash: Box<[u8]>,
        password: String,
    ) -> Result<PasswordState, Report<PasswordError>> {
        let password_data = rmp_serde::from_slice::<Password>(&password_hash)
            .map_err(|_| PasswordError("Failed to deserialize password hash".to_string()))?;

        match password_data {
            Password::Version1 { argon2 } => {
                let parsed_hash = PasswordHash::new(&argon2)
                    .map_err(|_| PasswordError("Failed to parse password hash".to_string()))?;

                match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
                    Ok(_) => Ok(PasswordState::Valid),
                    Err(_) => Ok(PasswordState::Invalid),
                }
            }
        }
    }

    pub fn encode_to_msg_pack(&self) -> Result<Box<[u8]>, Report<PasswordError>> {
        Ok(rmp_serde::to_vec_named(self)
            .map_err(|e| {
                PasswordError(format!(
                    "Failed to serialize password hash: {}",
                    e.to_string()
                ))
            })?
            .into())
    }
}
