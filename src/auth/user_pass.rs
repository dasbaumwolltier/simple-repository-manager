use std::collections::HashMap;
use std::str::FromStr;
use async_trait::async_trait;
use log::error;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::tokio::task;
use crate::auth::AuthenticationProvider;
use crate::config::{Config, PasswordType};
use crate::enclose;
use crate::utils::to_io_error;

pub struct UserPassAuthenticationProvider {
    users: HashMap<String, (String, PasswordType)>,

    // #[cfg(feature = "blake3")]
    blake3_hashes: HashMap<String, blake3::Hash>,
}

impl UserPassAuthenticationProvider {
    pub fn new(config: &Config) -> Self {
        let mut users = HashMap::new();
        let mut blake3_hashes = HashMap::new();
        let user_config = &config.users;

        user_config.iter()
            .for_each(|c| { users.insert(c.username.clone(), (c.password.clone(), c.password_type)); });

        user_config.iter()
            .filter(|c| c.password_type == PasswordType::Blake3)
            .for_each(|c| { blake3_hashes.insert(c.username.clone(), blake3::Hash::from_str(c.password.as_str()).unwrap()); });

        Self {
            users, blake3_hashes
        }
    }
}

#[async_trait]
impl AuthenticationProvider for UserPassAuthenticationProvider {
    async fn authenticate(&self, username: Option<String>, password: Option<String>) -> Result<bool, Custom<String>> {
        if username.is_none() {
            return Ok(false);
        }

        if password.is_none() {
            return Err(Custom(Status::Unauthorized, String::from("No password given!")));
        }

        let user_id = username.unwrap();
        let password = password.unwrap();

        let (hash, password_type) = match self.users.get(&user_id) {
            Some(hash) => hash.clone(),
            None => return Err(Custom(Status::Unauthorized, String::from("Username or password mismatch!")))
        };

        let blake3_hash = self.blake3_hashes.get(&user_id).cloned();

        let res = task::spawn_blocking(enclose! { (password, hash, blake3_hash, password_type) move || match password_type {
            PasswordType::BCrypt => to_io_error(bcrypt::verify(password, &hash)),
            PasswordType::Argon2 => to_io_error(argon2::verify_encoded(&hash, password.as_bytes())),
            PasswordType::Blake3 => Ok(blake3::hash(password.as_bytes()) == blake3_hash.unwrap())
        }}).await.map_err(|e| Custom(Status::InternalServerError, format!("{}", e)))?;

        match res {
            Ok(r) =>
                if r {
                    Ok(true)
                } else {
                    Err(Custom(Status::Unauthorized, String::from("Username or password mismatch!")))
                },
            Err(e) => {
                error!("Could not verify password! {}", e);
                return Err(Custom(Status::Unauthorized, String::from("Username or password mismatch!")));
            }
        }
    }
}