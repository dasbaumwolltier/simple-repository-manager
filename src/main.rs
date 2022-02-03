use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use actix_web::web::Data;
use clap::Arg;
use log::{error};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use crate::config::RepositoryConfig;
use crate::repository::file::FileRepository;
use crate::repository::RepositoryProvider;

mod config;
mod repository;
mod rest;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let matches = clap::App::new("Simple Repository Manager")
        .version("0.1")
        .author("Gabriel Guldner <gabriel@guldner.eu>")
        .about("Remote Backend for PowerDNS providing an easy way to configure health checks and load balance based on these.")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .required(true)
            .takes_value(true)
        ).arg(Arg::new("verbose")
        .short('v')
        .multiple_occurrences(true)
        .long("verbose")
        .required(false)
        .takes_value(false)
    ).get_matches();

    let level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    ).map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

    let config_file = match File::open(matches.value_of("config").unwrap()) {
        Ok(config_file) => config_file,
        Err(e) => {
            log::error!("Could not open config.yaml: {}", e);
            return Err(e);
        }
    };

    let reader = BufReader::new(config_file);
    let config: config::Config = match serde_yaml::from_reader(reader) {
        Ok(config) => config,
        Err(e) => {
            error!("Could not deserialize config: {}", e);
            return Err(std::io::Error::new(ErrorKind::Other, e));
        }
    };

    let mut providers: HashMap<String, Arc<dyn RepositoryProvider + Send + Sync>> = HashMap::new();

    for repository in config.repositories {
        match repository {
            RepositoryConfig::File { name, path, permissions } => {
                let repository = FileRepository::new(path, &permissions, &config.users);
                providers.insert(name, Arc::new(repository));
            }
        }
    }

    let provider_data = Data::new(providers);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "PUT"]);

        App::new()
            .wrap(cors)
            .app_data(Data::clone(&provider_data))
            .service(rest::retrieve)
            .service(rest::upload)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
