#![recursion_limit="256"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
extern crate slog_term;
extern crate slog_async;
extern crate slog_perf;
extern crate slog_config;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate bodyparser;
extern crate hyper;
extern crate hyper_native_tls;
extern crate config;
extern crate urlencoded;
extern crate serde;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate base64;
extern crate serde_json;
extern crate chrono;
extern crate jsonwebtoken as jwt;

use std::fs;
use std::io;
use std::io::{Write, Read};
use std::collections::HashSet;

use iron::prelude::*;
use mount::Mount;
use staticfile::Static;
use hyper_native_tls::{NativeTlsClient, NativeTlsServer};
use hyper::client::Client;
use hyper::net::HttpsConnector;
use config::{Config, File, FileFormat};
use slog::{Logger, Drain};

mod request;
mod login;
mod errors;
mod providers;
mod google;
mod auth;
mod models;

use errors::*;

fn main() {
    let mut config = Config::new();
    config.merge(File::new("config.toml", FileFormat::Toml)).expect("Failed to open config file!");

    let log = match build_drain(&config) {
        Ok(drain) => slog::Logger::root(
            std::sync::Arc::new(slog_async::Async::new(drain.fuse()).build().fuse()), o!()
        ),
        Err(err) => {
            eprintln!("Failed to create drain so will use Discard! {}", err);
            slog::Logger::root(
                std::sync::Arc::new(slog_async::Async::new(slog::Discard).build().fuse()), o!())
        }
    };

    let mut paths = HashSet::new();
    paths.insert("/auth".to_string());
    paths.shrink_to_fit();

    match start_server(&log, &config, paths) {
        Ok(_) => info!(log, "Successfully started the server"),
        Err(err) => error!(log, "Failed to start server! {}", err)
    }
}

fn build_auth(config: &Config, paths: HashSet<String>) -> Result<providers::AuthProvider> {
    let client = NativeTlsClient::new().map_err(|err| Error::from(ErrorKind::ClientTlsError(err)))?;
    let client = Client::with_connector(HttpsConnector::new(client));
    providers::AuthProvider::new(client, paths)
}

fn start_server(log: &Logger, config: &Config, paths: HashSet<String>) -> Result<iron::Listening> {
    let ssl = build_ssl(config)?;
    debug!(log, "Initialised SSL");
    let auth_provider = build_auth(config, paths)?;
    debug!(log, "Initialised Authentication");
    let db_provider = providers::DatabaseProvider::new(config)?;
    debug!(log, "Initialised Database");

    let mut mount = Mount::new();
    mount.mount("/", Static::new("web/"))
        .mount("/request", request::RequestHandler{})
        .mount("/login", login::LoginHandler::new());
    let mut chain = Chain::new(mount);
    chain.link_before(providers::LogProvider::new(log.new(o!())))
        .link_before(providers::MonitoringProvider {})
        .link_before(auth_provider)
        .link_before(db_provider);
    chain.link_after(providers::MonitoringProvider {})
        .link_after(providers::ErrorCapture {});
    build_iron(config, chain, ssl)
}

fn build_ssl(config: &Config) -> Result<NativeTlsServer> {
    let ssl_table = config.get_table("ssl")
        .map_err(|err| Error::from(ErrorKind::Config(err)))?;

    let identity = ssl_table.get("identity")
        .ok_or_else(|| ErrorKind::MissingConfigValueTable("identity".to_string(), "ssl".to_string()))
        .and_then(|v| v.clone().into_str().map_err(ErrorKind::Config))
        .map_err(Error::from)?;

    if !std::path::Path::new(&identity).exists() {
        bail!(ErrorKind::IdentityFileNotExistError(identity))
    }

    let pass = ssl_table.get("pass")
        .ok_or_else(|| ErrorKind::MissingConfigValueTable("pass".to_string(), "ssl".to_string()))
        .and_then(|v| v.clone().into_str().map_err(ErrorKind::Config))
        .map_err(Error::from)?;

    NativeTlsServer::new(&identity, &pass)
        .map_err(|err| Error::from(ErrorKind::TLS(err)))
}

fn build_iron<H: iron::middleware::Handler>(config: &Config, handler: H, ssl: NativeTlsServer) -> Result<iron::Listening> {
    let table = config.get_table("server")
        .map_err(|err| Error::from(ErrorKind::Config(err)))?;

    let ip = table.get("ip")
        .ok_or_else(|| ErrorKind::MissingConfigValueTable("ip".to_string(), "server".to_string()))
        .and_then(|v| v.clone().into_str().map_err(ErrorKind::Config))
        .map_err(Error::from)?;

    let port = table.get("port")
        .ok_or_else(|| ErrorKind::MissingConfigValueTable("port".to_string(), "server".to_string()))
        .and_then(|v| v.clone().into_int().map_err( ErrorKind::Config))
        .map_err(Error::from)?;

    Iron::new(handler).https((ip.as_ref(), port as u16), ssl).map_err(|err|Error::from(ErrorKind::HTTP(err)))
}

fn build_drain(config: &Config) -> Result<slog_config::Drain> {
    let path = config.get_str("logger_config")
        .map_err(|err| Error::from(ErrorKind::Config(err)))?;
    let mut file = fs::File::open(path)
        .map_err(|err| Error::from(ErrorKind::IO(err)))?;

    let mut data = String::new();
    let _ = file.read_to_string(&mut data)
        .map_err(|err| Error::from(ErrorKind::IO(err)))?;
    slog_config::from_config(&data)
        .map_err(|err| Error::from(ErrorKind::LoggerConfig(err)))
}
