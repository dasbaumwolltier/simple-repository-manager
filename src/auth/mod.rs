use async_trait::async_trait;
use rocket::response::status::Custom;

pub mod user_pass;

#[async_trait]
pub trait AuthenticationProvider {
    async fn authenticate(&self, username: Option<String>, password: Option<String>) -> Result<bool, Custom<String>>;
}