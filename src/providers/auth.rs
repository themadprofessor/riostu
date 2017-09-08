use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use iron::prelude::*;
use iron::BeforeMiddleware;
use hyper::Client;
use hyper::status::StatusCode;
use base64;
use bodyparser::Struct;
use serde_json;
use jwt::{self, Validation, Header};

use errors::*;
use google::{CachedKeys, CachedDiscovery, Key};
use models::User;

pub struct Auth {
    paths: HashSet<String>,
    keys: Arc<Mutex<CachedKeys>>,
    discovery: Arc<Mutex<CachedDiscovery>>,
    client: Client
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleToken {
    iss: String,
    at_hash: String,
    email_verified: String,
    sub: String,
    azp: String,
    email: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct UserData {
    user: User,
    jwt: String
}

impl Auth {
    pub fn new(client: Client, paths: HashSet<String>) -> Result<Auth> {
        CachedDiscovery::new(&client)
            .and_then(|mut discovery| discovery.discovery(&client)
                .and_then(|disc| CachedKeys::new(&client, disc))
                    .map(|keys| Auth {
                        client,
                        paths,
                        keys: Arc::new(Mutex::new(keys)),
                        discovery: Arc::new(Mutex::new(discovery))
                    })

            ).map_err(|err| Error::from(ErrorKind::GoogleError(err))
        )
    }
}

impl BeforeMiddleware for Auth {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        if self.paths.contains(req.url.as_ref().path()) {
            self.discovery.lock()
                .map_err(|err| Error::from(ErrorKind::PoisonError(format!("{}", err), "CachedDiscovery".to_string())))
                .and_then(|mut cached_disc| cached_disc.discovery(&self.client)
                    .map_err(|err| Error::from(ErrorKind::GoogleError(err)))
                    .and_then(|disc| self.keys.lock()
                        .map_err(|err| Error::from(ErrorKind::PoisonError(format!("{}", err), "CachedKeys".to_string())))
                        .and_then(|mut cached| cached.keys(&self.client, disc)
                            .map_err(|err| Error::from(ErrorKind::GoogleError(err)))
                            .and_then(|keys| req.get::<Struct<UserData>>()
                                .map_err(|err| Error::from(ErrorKind::RequestBody2Error(err)))
                                .and_then(|data| data.ok_or_else(|| Error::from(ErrorKind::MissingRequestError)))
                                .and_then(|user_data| get_key(&user_data.jwt, keys)
                                    .and_then(|key| base64::decode(&key.n)
                                        .map_err(ErrorKind::Base64Error)
                                        .and_then(|secret| {
                                            let mut validation = Validation::default();
                                            validation.iss = Some("accounts.google.com".to_string());
                                            jwt::decode::<GoogleToken>(&user_data.jwt, &secret, &validation)
                                                .map_err(ErrorKind::JwtError)
                                        }).map(|_| ())
                                    ).map_err(Error::from)
                                ).map_err(Error::from)
                            )
                        )
                    )
                ).map_err(|err| IronError {
                error: Box::new(Error::from(err)),
                response: Response::with((StatusCode::ImATeapot, "Teapot"))
            })
        } else {
            Ok(())
        }
    }
}

fn get_key<'a>(token: &str, keys: &'a [Key]) -> ::std::result::Result<&'a Key, ErrorKind> {
    serde_json::from_str::<Header>(token.split('.').next().unwrap())
        .map_err(ErrorKind::JsonError)
        .and_then(|head| head.kid.ok_or_else(|| ErrorKind::MissingKidGoogleTokenError))
        .and_then(|kid| keys.iter()
            .find(|key| key.kid == kid)
            .ok_or_else(|| ErrorKind::NoValidKeyGoogleError))
}

