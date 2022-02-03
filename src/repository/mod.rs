use std::path::{Path, PathBuf};
use actix_web::Error;
use crate::config::Permission;

pub mod file;

pub trait RepositoryProvider {
    fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()>;
    fn is_permitted(&self, user_id: Option<String>, password: Option<String>, required: &Permission) -> Result<(), Error>;
}