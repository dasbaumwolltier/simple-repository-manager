use std::collections::HashMap;
use std::path::{PathBuf};
use actix_web::Error;
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use bcrypt::verify;
use log::error;
use path_clean::PathClean;
use crate::config::{Permission, PermissionConfig, UserConfig};
use crate::repository::RepositoryProvider;

#[derive(Clone)]
pub struct FileRepository {
    base_path: String,
    permissions: HashMap<String, Permission>,
    users: HashMap<String, String>,
    anonymous: Permission,
}

impl RepositoryProvider for FileRepository {
    fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()> {
        let full_path = PathBuf::from(&self.base_path).join(path).clean();

        if !full_path.starts_with(&self.base_path) {
            return Err(());
        }

        return Ok(full_path);
    }

    fn is_permitted(&self, user_id: Option<String>, password: Option<String>, required: &Permission) -> Result<(), Error> {
        if user_id.is_none() {
            return if self.anonymous.is_permitted(&required) {
                Ok(())
            } else {
                Err(ErrorForbidden("Anonymous user not permitted!"))
            };
        }

        if password.is_none() {
            return Err(ErrorUnauthorized("No password given!"));
        }

        let user_id = user_id.unwrap();
        let password = password.unwrap();

        let permission = match self.permissions.get(&user_id) {
            Some(permission) => permission,
            None => return Err(ErrorUnauthorized("Username or password mismatch!"))
        };

        let hash = match self.users.get(&user_id) {
            Some(hash) => hash,
            None => return Err(ErrorUnauthorized("Username or password mismatch!"))
        };

        match verify(password, hash) {
            Ok(m) =>
                if m {
                    Ok(())
                } else {
                    Err(ErrorUnauthorized("Username or password mismatch!"))
                },
            Err(e) => {
                error!("Could not verify password! {}", e);
                return Err(ErrorUnauthorized("Username or password mismatch!"));
            }
        }
    }
}

impl FileRepository {
    pub fn new(path: String, permissions: &Vec<PermissionConfig>, user_config: &Vec<UserConfig>) -> Self {
        let mut anonymous = Permission::Read;
        let mut map = HashMap::new();
        let mut users = HashMap::new();

        permissions.iter()
            .filter(|c| !c.anonymous)
            .map(|c| (c.username.as_ref().unwrap().clone(), c.permission))
            .for_each(|c| {
                map.insert(c.0, c.1);
            });

        user_config.iter()
            .for_each(|c| { users.insert(c.username.clone(), c.password.clone()); });

        let anonymous_config = permissions.iter()
            .filter(|c| c.anonymous)
            .last();

        if let Some(config) = anonymous_config {
            anonymous = config.permission;
        }

        Self {
            base_path: path,
            permissions: map,
            anonymous,
            users,
        }
    }
}