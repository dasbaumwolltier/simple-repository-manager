use std::collections::HashMap;
use std::fs::{copy, create_dir_all, File};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use log::{error, trace};
use path_clean::PathClean;
use rocket::{Data, Either, FromForm, get, put, State};
use rocket::config::pretty_print_error;
use rocket::data::ToByteUnit;
use rocket::fs::{NamedFile, TempFile};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::response::status::{Custom, NotFound};
use rocket::tokio::{fs, io, task};
use rocket_basicauth::BasicAuth;
use crate::{enclose, Repository};
use crate::config::Permission;
use crate::constants::FILE_NOT_FOUND;

#[derive(FromForm)]
pub struct Upload<'r> {
    file: Vec<TempFile<'r>>
}

#[get("/<repository>/<path..>")]
pub async fn retrieve(repository: String, path: PathBuf, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn Repository + Send + Sync>>>) -> impl Responder<'_, '_> {
    let provider = match providers.get(&repository) {
        Some(provider) => provider.to_owned(),
        None => return Err(Custom(Status::NotFound, String::from("Could not find repository!")))
    };

    authenticate(provider.to_owned(), auth, &Permission::Read).await?;

    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = get_file(provider, &file).await?;
    if file.is_dir() {
        // return Ok(Either::Left(Directory::new(file, PathBuf::new())));
        return Ok(Either::Left(""))
    }

    match NamedFile::open(file).await {
        Ok(file) => Ok(Either::Right(file)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(Custom(Status::NotFound, String::from(FILE_NOT_FOUND))),
            _ => Err(Custom(Status::InternalServerError, format!("{:?}", e)))
        }
    }
}

// #[put("/<repository>", data = "<data>")]
// pub async fn upload(data: Data<'_>, cont_type: &ContentType, repository: String, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> Result<Status, Custom<String>> {
//     upload_path(data, cont_type, repository, None, auth, providers).await
// }

#[put("/<repository>/<path..>", data = "<data>")]
pub async fn upload(mut data: Data<'_>, cont_type: &ContentType, repository: String, path: Option<PathBuf>, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn Repository + Send + Sync>>>) -> Result<Status, Custom<String>> {
    let provider = match providers.get(&repository) {
        Some(provider) => provider.to_owned(),
        None => return Err(Custom(Status::NotFound, String::from(FILE_NOT_FOUND)))
    };

    authenticate(provider.to_owned(), auth, &Permission::Read).await?;

    let path = path.unwrap_or(PathBuf::new());
    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = get_file(provider, &file).await?;
    if !file.exists() {
        let parent = match file.parent() {
            Some(parent) => parent.clone(),
            None => {
                return Err(Custom(Status::BadRequest, String::from("Invalid path!")));
            }
        };

        let parent = PathBuf::from(parent);
        task::spawn_blocking(enclose! { (parent) move || create_dir_all(parent) }).await;
    }

    process_upload(file, data).await?;
    Ok(Status::Created)
}

async fn authenticate(provider: Arc<dyn Repository + Send + Sync>, auth: Option<BasicAuth>, required_permission: &Permission) -> Result<(), Custom<String>> {
    let is_anonymous = provider.get_auth_provider().authenticate(
        auth.as_ref().map(|b| b.username.to_owned()),
        auth.as_ref().map(|b| b.password.to_owned()),
    ).await?;

    provider.is_authorized(
        auth.as_ref().map(|b| b.username.to_owned()),
        required_permission,
        is_anonymous
    ).await
}

async fn get_file(provider: Arc<dyn Repository + Send + Sync>, file: &PathBuf) -> Result<PathBuf, Custom<String>> {
    match provider.get_file(&file).await {
        Ok(file) => Ok(file),
        Err(e) => {
            error!("Invalid path: {}!", file.to_str().unwrap());
            Err(Custom(Status::BadRequest, String::from("Invalid path!")))
        }
    }
}

async fn process_upload(file: PathBuf, data: Data<'_>) -> Result<(), Custom<String>> {
    let mut stream = data.open(1i32.gibibytes());
    let mut file = match fs::File::create(&file).await {
        Ok(file) => file,
        Err(e) => {
            error!("Could not open file: {}!", e);
            return Err(Custom(Status::InternalServerError, String::new()))
        }
    };

    trace!("Uploaded file has size {}", stream.hint());
    match io::copy(&mut stream, &mut file).await {
        Err(e) => {
            error!("Could not write file: {}!", e);
            return Err(Custom(Status::InternalServerError, String::new()))
        },
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::auth::AuthenticationProvider;

    use mockall::mock;
    use rocket::tokio;

    mock! {
        pub Provider {}

        #[async_trait::async_trait]
        impl Repository for Provider {
            async fn get_file(&self, path: &PathBuf) -> Result<PathBuf, ()>;
            async fn is_authorized(&self, user_id: Option<String>, required: &Permission, is_anonymous: bool) -> Result<(), Custom<String>>;

            fn get_auth_provider(&self) -> Arc<dyn AuthenticationProvider + Send + Sync>;
        }
    }

    mock! {
        pub AuthProvider {}

        #[async_trait::async_trait]
        impl AuthenticationProvider for AuthProvider {
            async fn authenticate(&self, username: Option<String>, password: Option<String>) -> Result<bool, Custom<String>>;
        }
    }

    #[tokio::test]
    async fn test_authenticate() {
        let auth = Some(BasicAuth {
            username: String::from("user"),
            password: String::from("password")
        });

        let mut auth_provider_mock = MockAuthProvider::new();
        let mut provider_mock = MockProvider::new();

        auth_provider_mock
            .expect_authenticate()
            .return_const(Ok(true));

        provider_mock
            .expect_is_authorized()
            .returning(|_, _, _| Ok(()));

        let auth_provider = Arc::new(auth_provider_mock);
        provider_mock.expect_get_auth_provider()
            .returning(enclose! { (auth_provider) move || auth_provider.clone() });

        let provider = Arc::new(provider_mock);
        let res = authenticate(provider, auth, &Permission::Read).await;

        assert!(res.is_ok());

        let auth = Some(BasicAuth {
            username: String::from("user"),
            password: String::from("password")
        });

        let mut auth_provider_mock = MockAuthProvider::new();
        let mut provider_mock = MockProvider::new();

        auth_provider_mock
            .expect_authenticate()
            .return_const(Ok(false));

        provider_mock
            .expect_is_authorized()
            .returning(|_, _, _| Ok(()));

        let auth_provider = Arc::new(auth_provider_mock);
        provider_mock.expect_get_auth_provider()
            .returning(enclose! { (auth_provider) move || auth_provider.clone() });

        let provider = Arc::new(provider_mock);
        let res = authenticate(provider, auth, &Permission::Read).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_file() {
        let mut mock = MockProvider::new();

        mock.expect_get_file()
            .times(1)
            .returning(|p| Ok(PathBuf::from("/tmp").join(p)));

        let provider = Arc::new(mock);
        let res = get_file(provider, &PathBuf::from("test.txt")).await;

        assert!(res.is_ok());
        assert_eq!(&res.as_ref().unwrap().to_str().unwrap(), &"/tmp/test.txt");
    }

    #[tokio::test]
    async fn test_get_file_error() {
        let mut mock = MockProvider::new();

        mock.expect_get_file()
            .times(1)
            .return_const(Err(()));

        let provider = Arc::new(mock);
        let res = get_file(provider, &PathBuf::from("/test.txt")).await;

        assert!(res.is_err());
        assert_eq!(&res.as_ref().unwrap_err().0, &Status::BadRequest);
        assert_eq!(&res.as_ref().unwrap_err().1, "Invalid path!");
    }
}