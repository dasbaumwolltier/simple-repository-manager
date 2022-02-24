use std::path::{PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use rocket::response::status::Custom;
use crate::auth::AuthenticationProvider;
use crate::config::Permission;

pub mod file;

#[async_trait]
pub trait Repository {
    async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()>;
    async fn is_authorized(&self, user_id: Option<String>, required: &Permission, is_anonymous: bool) -> Result<(), Custom<String>>;

    fn get_auth_provider(&self) -> Arc<dyn AuthenticationProvider + Send + Sync>;
}