use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use async_trait::async_trait;
use log::error;
use path_clean::PathClean;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::tokio::task;
use crate::config::{PasswordType, Permission, PermissionConfig, UserConfig};
use crate::enclose;
use crate::repository::RepositoryProvider;
use crate::utils::to_io_error;

#[derive(Clone)]
pub struct FileRepository {
    base_path: String,
    permissions: HashMap<String, Permission>,
    users: HashMap<String, (String, PasswordType)>,
    anonymous: Permission,

    // #[cfg(feature = "blake3")]
    blake3_hashes: HashMap<String, blake3::Hash>,
}

#[async_trait]
impl RepositoryProvider for FileRepository {
    async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()> {
        let full_path = PathBuf::from(&self.base_path).join(path).clean();

        if !full_path.starts_with(&self.base_path) {
            return Err(());
        }

        return Ok(full_path);
    }

    async fn is_permitted(&self, user_id: Option<String>, password: Option<String>, required: &Permission) -> Result<(), Custom<String>> {
        if user_id.is_none() {
            return if self.anonymous.is_permitted(&required) {
                Ok(())
            } else {
                Err(Custom(Status::Forbidden, String::from("Anonymous user not permitted!")))
            };
        }

        if password.is_none() {
            return Err(Custom(Status::Unauthorized, String::from("No password given!")));
        }

        let user_id = user_id.unwrap();
        let password = password.unwrap();

        let permission = match self.permissions.get(&user_id) {
            Some(permission) => permission,
            None => return Err(Custom(Status::Unauthorized, String::from("Username or password mismatch!")))
        };

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
                    Ok(())
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

impl FileRepository {
    pub fn new(path: String, permissions: &Vec<PermissionConfig>, user_config: &Vec<UserConfig>) -> Self {
        let mut anonymous = Permission::Read;
        let mut map = HashMap::new();
        let mut users = HashMap::new();
        let mut blake3_hashes = HashMap::new();

        permissions.iter()
            .filter(|c| !c.anonymous)
            .map(|c| (c.username.as_ref().unwrap().clone(), c.permission))
            .for_each(|c| {
                map.insert(c.0, c.1);
            });

        user_config.iter()
            .for_each(|c| { users.insert(c.username.clone(), (c.password.clone(), c.password_type)); });

        user_config.iter()
            .filter(|c| c.password_type == PasswordType::Blake3)
            .for_each(|c| { blake3_hashes.insert(c.username.clone(), blake3::Hash::from_str(c.password.as_str()).unwrap()); });

        let anonymous_config = permissions.iter()
            .filter(|c| c.anonymous)
            .last();

        if let Some(config) = anonymous_config {
            anonymous = config.permission;
        }

        Self {
            base_path: path,
            permissions: map,
            blake3_hashes,
            anonymous,
            users,
        }
    }
}
