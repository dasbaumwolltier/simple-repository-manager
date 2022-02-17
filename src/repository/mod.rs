use std::path::{PathBuf};
use async_trait::async_trait;
use rocket::response::status::Custom;
use crate::config::Permission;

pub mod file;

#[async_trait]
pub trait Repository {
    async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()>;
    async fn is_permitted(&self, user_id: Option<String>, password: Option<String>, required: &Permission) -> Result<(), Custom<String>>;
}