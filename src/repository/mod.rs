use std::path::{PathBuf};
use actix_web::Error;
use async_trait::async_trait;
use crate::config::Permission;

pub mod file;

#[async_trait]
pub trait RepositoryProvider {
    async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()>;
    async fn is_permitted(&self, user_id: Option<String>, password: Option<String>, required: &Permission) -> Result<(), Error>;
}