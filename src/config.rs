use serde_derive::Deserialize;
use crate::config::Permission::Write;

#[derive(Deserialize)]
pub struct Config {
    pub repositories: Vec<RepositoryConfig>,
    pub users: Vec<UserConfig>
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserConfig {
    pub username: String,
    pub password_type: PasswordType,
    pub password: String,
}

#[derive(Copy, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PasswordType {
    BCrypt,
    Argon2,
    Blake3
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RepositoryConfig {
    File {
        name: String,
        path: String,
        permissions: Vec<PermissionConfig>,
        #[serde(default)]
        authentication: AuthenticationConfig
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AuthenticationConfig {
    Yaml
}

#[derive(Clone, Deserialize)]
pub struct PermissionConfig {
    pub username: Option<String>,
    pub permission: Permission,

    #[serde(default)]
    pub anonymous: bool
}

#[derive(Copy, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    None,
    Read,
    Write
}

impl Permission {
    pub fn is_permitted(&self, required: &Permission) -> bool {
        match required {
            Permission::Write => self == &Write,
            Permission::Read => self == &Write || self == &Permission::Read,
            Permission::None => true
        }
    }
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        AuthenticationConfig::Yaml
    }
}