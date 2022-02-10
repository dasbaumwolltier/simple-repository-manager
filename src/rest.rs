use std::collections::HashMap;
use std::fs::{copy, create_dir_all, File};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
// use actix_files::{NamedFile};
// use actix_multipart::Multipart;
// use actix_web::{Either, HttpResponse, Responder, web};
// use actix_web::{get, put};
// use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
// use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use log::{error, trace};
use path_clean::PathClean;
use rocket::{Data, Either, FromForm, get, put, State};
use rocket::data::ToByteUnit;
use rocket::fs::{NamedFile, TempFile};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::response::status::{Custom, NotFound};
use rocket::tokio::{fs, io, task};
use rocket_basicauth::BasicAuth;
use crate::{enclose, RepositoryProvider};
use crate::config::Permission;

// pub async fn validator(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, Error> {
//     req.extensions_mut().insert(credentials);
//     Ok(req)
// }

#[derive(FromForm)]
pub struct Upload<'r> {
    file: Vec<TempFile<'r>>
}

#[get("/<repository>/<path..>")]
pub async fn retrieve(repository: String, path: PathBuf, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> impl Responder<'_, '_> {
    let provider = match providers.get(&repository) {
        Some(provider) => provider,
        None => return Err(Custom(Status::NotFound, String::from("Could not find repository!")))
    };

    match provider.is_permitted(
        auth.as_ref().map(|b| b.username.to_owned()),
        auth.as_ref().map(|b| b.password.to_owned()),
       &Permission::Read
    ).await {
        Err(e) => return Err(e),
        _ => {}
    }

    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = match provider.get_file(&file).await {
        Ok(file) => file,
        Err(e) => {
            error!("Invalid path for repo {}: {}!", repository, file.to_str().unwrap());
            return Err(Custom(Status::BadRequest, String::from("Invalid path!")));
        }
    };

    if file.is_dir() {
        // return Ok(Either::Left(Directory::new(file, PathBuf::new())));
        return Ok(Either::Left(""))
    }

    match NamedFile::open(file).await {
        Ok(file) => Ok(Either::Right(file)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(Custom(Status::NotFound, String::from("Could not find file!"))),
            _ => Err(Custom(Status::InternalServerError, format!("{:?}", e)))
        }
    }
}

// #[put("/<repository>", data = "<data>")]
// pub async fn upload(data: Data<'_>, cont_type: &ContentType, repository: String, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> Result<Status, Custom<String>> {
//     upload_path(data, cont_type, repository, None, auth, providers).await
// }

#[put("/<repository>/<path..>", data = "<data>")]
pub async fn upload(mut data: Data<'_>, cont_type: &ContentType, repository: String, path: Option<PathBuf>, auth: Option<BasicAuth>, providers: &State<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> Result<Status, Custom<String>> {
    let provider = match providers.get(&repository) {
        Some(provider) => provider,
        None => return Err(Custom(Status::NotFound, String::from("Could not find repository!")))
    };

    match provider.is_permitted(
        auth.as_ref().map(|b| b.username.to_owned()),
        auth.as_ref().map(|b| b.password.to_owned()),
        &Permission::Write
    ).await {
        Err(e) => return Err(e),
        _ => {}
    }

    let path = path.unwrap_or(PathBuf::new());
    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = match provider.get_file(&file).await {
        Ok(file) => file,
        Err(e) => {
            error!("Invalid path for repo {}: {}!", repository, file.to_str().unwrap());
            return Err(Custom(Status::BadRequest, String::from("Invalid path!")));
        }
    };

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