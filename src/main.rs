use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use clap::Arg;
use log::{error};
use rocket::{launch, routes};
use simplelog::{ColorChoice, TerminalMode, TermLogger};
use crate::config::RepositoryConfig;
use crate::repository::file::FileRepository;
use crate::repository::Repository;

mod config;
mod repository;
mod rest;
mod utils;

#[launch]
fn launch() -> _ {
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
        ).arg(Arg::new("host")
            .short('h')
            .long("host")
            .value_name("HOST")
            .default_missing_value("127.0.0.1")
        ).arg(Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .default_missing_value("8080")
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
        simplelog::Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    ).map_err(|e| std::io::Error::new(ErrorKind::Other, e));

    let port = match u16::from_str(matches.value_of("port").unwrap()) {
        Ok(port) => port,
        Err(e) => {
            error!("Could not parse port: {}!", e);
            exit(1);
        }
    };

    let host = matches.value_of("host").unwrap_or("127.0.0.1");

    let config_file = match File::open(matches.value_of("config").unwrap()) {
        Ok(config_file) => config_file,
        Err(e) => {
            log::error!("Could not open config.yaml: {}", e);
            exit(1);
        }
    };

    let reader = BufReader::new(config_file);
    let config: config::Config = match serde_yaml::from_reader(reader) {
        Ok(config) => config,
        Err(e) => {
            error!("Could not deserialize config: {}", e);
            exit(1);
        }
    };

    let mut providers: HashMap<String, Arc<dyn Repository + Send + Sync>> = HashMap::new();

    for repository in config.repositories {
        match repository {
            RepositoryConfig::File { name, path, permissions } => {
                let repository = FileRepository::new(path, &permissions, &config.users);
                providers.insert(name, Arc::new(repository));
            }
        }
    }

    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", host))
        .merge(("level", "critical"));

    rocket::custom(figment)
        .manage(providers)
        .mount("/", routes![rest::retrieve, rest::upload])
}