use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use path_clean::PathClean;
use rocket::http::Status;
use rocket::response::status::Custom;
use crate::auth::AuthenticationProvider;
use crate::config::{Permission, PermissionConfig};
use crate::constants::FILE_NOT_FOUND;
use crate::repository::Repository;

#[derive(Clone)]
pub struct FileRepository {
    base_path: String,
    permissions: HashMap<String, Permission>,
    anonymous: Permission,
    auth_provider: Arc<dyn AuthenticationProvider + Send + Sync>
}

#[async_trait]
impl Repository for FileRepository {
    async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()> {
        let full_path = PathBuf::from(&self.base_path).join(path).clean();

        if !full_path.starts_with(&self.base_path) {
            return Err(());
        }

        return Ok(full_path);
    }

    async fn is_authorized(&self, user_id: Option<String>, required: &Permission, is_anonymous: bool) -> Result<(), Custom<String>> {
        if is_anonymous {
            return if self.anonymous.is_permitted(&required) {
                Ok(())
            } else {
                Err(Custom(Status::NotFound, String::from(FILE_NOT_FOUND)))
            };
        }

        let user_id = user_id.unwrap();
        let permission = match self.permissions.get(&user_id) {
            Some(permission) => permission,
            None => return Err(Custom(Status::Unauthorized, String::from("Username or password mismatch!")))
        };

        if permission.is_permitted(&required) {
            Ok(())
        } else {
            Err(Custom(Status::NotFound, String::from(FILE_NOT_FOUND)))
        }
    }

    fn get_auth_provider(&self) -> Arc<dyn AuthenticationProvider + Send + Sync> {
        self.auth_provider.clone()
    }
}

impl FileRepository {
    pub fn new(path: &String, permissions: &Vec<PermissionConfig>, auth_provider: Arc<dyn AuthenticationProvider + Send + Sync>) -> Self {
        let mut anonymous = Permission::Read;
        let mut map = HashMap::new();

        permissions.iter()
            .filter(|c| !c.anonymous)
            .map(|c| (c.username.as_ref().unwrap().clone(), c.permission))
            .for_each(|c| {
                map.insert(c.0, c.1);
            });

        let anonymous_config = permissions.iter()
            .filter(|c| c.anonymous)
            .last();

        if let Some(config) = anonymous_config {
            anonymous = config.permission;
        }

        Self {
            base_path: path.to_owned(),
            permissions: map,
            anonymous,
            auth_provider
        }
    }
}
