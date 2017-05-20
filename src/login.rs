use std::io::Read;
use std::sync::{Arc, RwLock};

use iron::prelude::*;
use iron::Handler;
use iron::status;
use urlencoded::{QueryMap, QueryResult, UrlEncodedBody};
use bodyparser::Struct;
use base64;
use serde_json;
use jwt::{self, Validation, Header, Algorithm};
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use providers::{DatabaseProvider, LogProvider};
use errors::*;
use models::User;
use google::{CachedKeys, CachedDiscovery, Key};

pub struct LoginHandler {
    keys: Arc<RwLock<CachedKeys>>,
    discovery: Arc<RwLock<CachedDiscovery>>,
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

impl LoginHandler {
    pub fn new(client: Client) -> Result<LoginHandler> {
        CachedDiscovery::new(&client)
            .map_err(ErrorKind::Google)
            .and_then(|cached_disc| cached_disc.discovery_opt()
                .ok_or_else(|| ErrorKind::Google(::google::error::ErrorKind::ExpiredDiscovery.into()))
                .and_then(|disc| CachedKeys::new(&client, &disc)
                    .map(|cached_keys| LoginHandler {
                        keys: Arc::new(RwLock::new(cached_keys)),
                        discovery: Arc::new(RwLock::new(cached_disc)),
                        client
                    }).map_err(ErrorKind::Google)))
            .map_err(Error::from)
    }

    fn get_keys<'a>(&'a self) -> Result<&'a Vec<Key>> {
        self.discovery.clone().read()
            .map_err(|err| ErrorKind::Poison(format!("{}", err), "CachedDiscovery".to_string()))
            .and_then(|cached_disc| cached_disc.discovery_opt()
                .ok_or_else(|| ErrorKind::Google(::google::error::ErrorKind::ExpiredDiscovery.into())))
            .or_else(|_| self.discovery.clone().write()
                .map_err(|err| ErrorKind::Poison(format!("{}", err), "CachedDiscovery".to_string()))
                .and_then(|cached_disc| cached_disc.discovery(&self.client).map_err(ErrorKind::Google)))
            .and_then(|discovery| self.keys.clone().read()
                .map_err(|err| ErrorKind::Poison(format!("{}", err), "CachedKeys".to_string()))
                .and_then(|cached_keys| cached_keys.keys_opt()
                    .ok_or_else(|| ErrorKind::Google(::google::error::ErrorKind::ExpiredKeys.into())))
                .or_else(|_| self.keys.clone().write()
                    .map_err(|err| ErrorKind::Poison(format!("{}", err), "CachedKeys".to_string()))
                    .and_then(|keys| keys.keys(&self.client, discovery).map_err(ErrorKind::Google))))
            .map_err(Error::from)
    }
}

impl Handler for LoginHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        println!("Request Headers: {}", req.headers);

        req.get::<Struct<UserData>>()
            .map_err(|err| Error::from(ErrorKind::RequestBody2(err)))
            .and_then(|data| data.ok_or_else(|| Error::from(ErrorKind::MissingRequest))
                .and_then(|user_data| req.extensions.get::<DatabaseProvider>()
                    .ok_or_else(|| Error::from(ErrorKind::MissingDatabaseConnection))
                    .and_then(|con| con.get().map_err(|err| Error::from(ErrorKind::PoolTimeout(err))))
                    .chain_err(|| ErrorKind::InternalServerError)
                    .and_then(|con| con.execute("INSERT INTO users VALUES ($1, $2, $3) on conflict (id) do nothing",
                                                &[&user_data.user.id, &user_data.user.name, &user_data.user.email])
                        .map_err(|err| Error::from(ErrorKind::Postgres(err)))
                        .chain_err(|| ErrorKind::InternalServerError)
                              /*.and_then(|con| con.query("SELECT token FROM tokens WHERE user_id=$1", &[&user_data.user.id]))*/)
                    .and_then(|_| self.get_keys()
                        .and_then(|keys| get_key(&user_data.jwt, &keys)
                            .and_then(|key| base64::decode(&key.n)
                                .map_err(ErrorKind::Base64)
                                .and_then(|secret| {
                                    let mut validation = Validation::default();
                                    jwt::decode::<GoogleToken>(&user_data.jwt, &secret, &validation)
                                        .map_err(ErrorKind::JWT)
                                }).map_err(Error::from))))))
        .map_err(|err| IronError {
            error: Box::new(err),
            response: Response::with((status::ImATeapot, "Teapot"))
                    }).map(|_| Response::with(status::Ok))
    }
}

fn get_key<'a>(token: &str, keys: &'a Vec<Key>) -> Result<&'a Key> {
    serde_json::from_str::<Header>(token.split('.').next().unwrap())
        .map_err(ErrorKind::JSON)
        .and_then(|head| head.kid.ok_or_else(|| ErrorKind::MissingKidGoogleToken))
        .and_then(|kid| keys.iter()
            .filter(|key| key.kid == kid)
            .next()
            .ok_or_else(|| ErrorKind::NoValidKeyGoogle))
        .map_err(Error::from)
}

fn build_user(map_res: QueryResult) -> Result<User> {
    map_res.map_err(|err| Error::from(ErrorKind::RequestBody(err)))
        .chain_err(|| ErrorKind::InternalServerError)
        .and_then(|mut map: QueryMap| {
            let id_res = map.remove("id")
                .ok_or_else(|| ErrorKind::MissingRequestData("id".to_string()))
                .and_then(|mut id| match id.len() {
                    1 => Ok(id.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("id".to_string(), 1))
                });
            let name_res = map.remove("name")
                .ok_or_else(|| ErrorKind::MissingRequestData("name".to_string()))
                .and_then(|mut name| match name.len() {
                    1 => Ok(name.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("name".to_string(), 1))
                });
            let email_res = map.remove("email")
                .ok_or_else(|| ErrorKind::MissingRequestData("email".to_string()))
                .and_then(|mut email| match email.len() {
                    1 => Ok(email.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("email".to_string(), 1))
                });

            id_res.and_then(
                |id| name_res.and_then(
                    |name| email_res.and_then(
                        |email| Ok(User{id, name, email}))))
            .map_err(Error::from)
            .chain_err(|| ErrorKind::BadRequest)
        })
}