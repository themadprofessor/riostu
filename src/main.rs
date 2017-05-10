#![recursion_limit="128"]

#[macro_use] extern crate slog;
extern crate slog_term;
extern crate slog_async;
extern crate slog_perf;
extern crate slog_config;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate hyper_native_tls;
extern crate config;
extern crate bodyparser;
extern crate urlencoded;
extern crate currency;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
extern crate serde;

use std::fs;
use std::io;
use std::path::Path;
use std::io::{Read, Write};

use iron::prelude::*;
use mount::Mount;
use staticfile::Static;
use hyper_native_tls::NativeTlsServer;
use config::{Config, File, FileFormat};
use slog::Drain;

mod request;
mod errors;
mod monitoring;
mod errorcapture;
mod logging;

use errors::*;

fn main() {
    let mut config = Config::new();
    config.merge(File::new("config.toml", FileFormat::Toml)).expect("Failed to open config file!");

    let log = match build_drain(&config) {
        Ok(drain) => slog::Logger::root(
            std::sync::Arc::new(slog_async::Async::new(drain.fuse()).build().fuse()), o!()
        ),
        Err(err) => {
            writeln!(io::stderr(), "Failed to create drain so will use Discard! {}", err);
            slog::Logger::root(
                std::sync::Arc::new(slog_async::Async::new(slog::Discard).build().fuse()), o!())
        }
    };


    match build_ssl(&config).and_then(|ssl| {
        let mut mount = Mount::new();
        mount.mount("/", Static::new("web/")).mount("/request", request::RequestHandler{});
        let mut chain = Chain::new(mount);
        chain.link_before(logging::LoggerMiddleware::new(log.new(o!())))
            .link_before(monitoring::MonitoringMiddleware{})
            .link_after(monitoring::MonitoringMiddleware{})
            .link_after(errorcapture::ErrorCapture{});
        build_iron(&config, chain, ssl)
    }) {
        Ok(_) => info!(log, "Successfully started the server"),
        Err(err) => error!(log, "Failed to start server! {}", err)
    }
}

fn build_ssl(config: &Config) -> Result<NativeTlsServer> {
    let ssl_identity = config.get_table("ssl")
        .ok_or(ErrorKind::MissingConfigValue("ssl".to_string()))
        .and_then(|table| table.get("identity")
            .and_then(|i| i.clone().into_str())
            .ok_or(ErrorKind::MissingConfigValueTable("identity".to_string(), "ssl".to_string())));
    let ssl_pass = config.get_table("ssl")
        .ok_or(ErrorKind::MissingConfigValue("ssl".to_string()))
        .and_then(|table| table.get("pass")
            .and_then(|p| p.clone().into_str())
            .ok_or(ErrorKind::MissingConfigValueTable("pass".to_string(), "ssl".to_string())));

    ssl_identity.and_then(
        |i| ssl_pass.and_then(
            |p| NativeTlsServer::new(i, &p)
                .map_err(|err| ErrorKind::TLS(err))))
        .map_err(|err| err.into())
}

fn build_iron<H: iron::middleware::Handler>(config: &Config, handler: H, ssl: NativeTlsServer) -> Result<iron::Listening> {
    let server_ip = config.get_table("server")
        .ok_or(ErrorKind::MissingConfigValue("server".to_string()))
        .and_then(|table| table.get("ip")
            .and_then(|i| i.clone().into_str())
            .ok_or(ErrorKind::MissingConfigValueTable("ip".to_string(), "server".to_string())));
    let server_port = config.get_table("server")
        .ok_or(ErrorKind::MissingConfigValue("server".to_string()))
        .and_then(|table| table.get("port")
            .ok_or(ErrorKind::MissingConfigValueTable("port".to_string(), "server".to_string()))
            .and_then(|p| p.clone()
                .into_int()
                .ok_or(ErrorKind::InvalidConfigType("port".to_string(), "integer".to_string()))));

    server_ip.and_then(|i: String|
        server_port.and_then(|port|
            Iron::new(handler).https((i.as_str(), port as u16), ssl)
                .map_err(|err| ErrorKind::HTTP(err))))
        .map_err(|err| err.into())
}

fn build_drain(config: &Config) -> Result<slog_config::Drain> {
    config.get_str("logger_config")
        .ok_or(ErrorKind::MissingConfigValue("logger_config".to_string()))
        .and_then(|p|
            fs::File::open(p).and_then(|mut file| {
                let mut s = String::new();
                file.read_to_string(&mut s).map(|_| s)
            }).map_err(|err| ErrorKind::IO(err)))
        .and_then(|s| slog_config::from_config(&s).map_err(|err| ErrorKind::LoggerConfig(err)))
        .map_err(Error::from)
}
