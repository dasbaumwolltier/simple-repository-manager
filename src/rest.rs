use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use actix_cors::CorsError::BadRequestHeaders;
use actix_files::{Directory, NamedFile};
use actix_multipart::Multipart;
use actix_web::{Either, HttpMessage, HttpResponse, Responder, web};
use actix_web::{get, put};
use actix_web::dev::{Extensions, ServiceRequest};
use actix_web::error::{Error, ErrorBadRequest, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use futures_util::future::err;
use futures_util::TryStreamExt;
use log::error;
use path_clean::PathClean;
use crate::{enclose, RepositoryProvider};
use crate::config::Permission;
use crate::config::Permission::Read;

// pub async fn validator(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, Error> {
//     req.extensions_mut().insert(credentials);
//     Ok(req)
// }

#[get("/{repository}/{path:[^?]+}")]
pub async fn retrieve(repository: web::Path<(String, String)>, auth: Option<web::Header<Authorization<Basic>>>, providers: web::Data<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> impl Responder {
    let (repository, path) = repository.into_inner();
    let provider = match providers.get(&repository) {
        Some(provider) => provider,
        None => return Err(ErrorNotFound("Could not find repository!"))
    };

    let scheme = auth.map(|a| a.into_inner().into_scheme());
    match provider.is_permitted(
        scheme.clone().map(|b| b.user_id().to_string()),
        scheme.map(|b| b.password().map(|p| p.to_string())).flatten(),
       &Permission::Read
    ) {
        Err(e) => return Err(e),
        _ => {}
    }

    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = match provider.get_file(&file) {
        Ok(file) => file,
        Err(e) => {
            error!("Invalid path for repo {}: {}!", repository, file.to_str().unwrap());
            return Err(ErrorBadRequest("Invalid path!"));
        }
    };

    if web::block(enclose! { (file) move || file.is_dir() }).await? {
        // return Ok(Either::Left(Directory::new(file, PathBuf::new())));
        return Ok(Either::Left(""))
    }

    match NamedFile::open(file) {
        Ok(file) => Ok(Either::Right(file)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(ErrorNotFound("Could not find file!")),
            _ => Err(ErrorInternalServerError(e))
        }
    }
}

#[put("/{repository}/{path:[^?]+}")]
pub async fn upload(mut content: Multipart, repository: web::Path<(String, String)>, auth: Option<web::Header<Authorization<Basic>>>, providers: web::Data<HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>>>) -> impl Responder {
    let (repository, path) = repository.into_inner();
    let provider = match providers.get(&repository) {
        Some(provider) => provider,
        None => return Err(ErrorNotFound("Could not find repository!"))
    };

    let scheme = auth.map(|a| a.into_inner().into_scheme());
    match provider.is_permitted(
        scheme.clone().map(|b| b.user_id().to_string()),
        scheme.map(|b| b.password().map(|p| p.to_string())).flatten(),
        &Permission::Write
    ) {
        Err(e) => return Err(e),
        _ => {}
    }

    let mut file = PathBuf::from(path).clean();
    if file.starts_with("../") {
        let string = file.to_str().unwrap();
        file = PathBuf::from(string.get(3..).unwrap());
    }

    let file = match provider.get_file(&file) {
        Ok(file) => file,
        Err(e) => {
            error!("Invalid path for repo {}: {}!", repository, file.to_str().unwrap());
            return Err(ErrorBadRequest("Invalid path!"));
        }
    };
    while let Some(mut field) = content.try_next().await? {
        let content_disposition = field.content_disposition();

        let filename = match content_disposition.get_filename() {
            Some(name) => name,
            None => return Err(ErrorBadRequest("Could not get filename!"))
        };

        if web::block(enclose! { (file) move || file.exists() }).await? {
            if web::block(enclose! { (file) move || file.is_file() }).await? {
                return Err(ErrorBadRequest("Could not create directory! File with the same name already exists!"))
            }
        } else {
            match web::block(enclose! { (file) move || create_dir_all(file) }).await? {
                Err(_) => return Err(ErrorInternalServerError("Could not create directory!")),
                _ => {}
            }
        }

        let file = file.clone().join(Path::new(&filename));
        let mut file = match web::block(move || File::create(file)).await? {
            Ok(file) => file,
            Err(_) => return Err(ErrorInternalServerError("Could not write file!"))
        };

        while let Some(chunk) = field.try_next().await? {
            file = web::block(move || file.write_all(&chunk).map(|_| file)).await??;
        }
    }

    Ok(HttpResponse::Created())
}