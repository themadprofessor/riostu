use std::io::Read;

use hyper::{header, Client};
use serde_json;
use chrono::{DateTime, UTC, Duration};

use super::error::*;
use super::discovery::Discovery;

#[derive(Deserialize, Debug)]
struct Keys {
    keys: Vec<Key>
}

#[derive(Deserialize, Debug)]
pub struct Key {
    pub kty: String,
    pub alg: String,
    #[serde(rename="use")]
    pub use_on: String,
    pub kid: String,
    pub n: String,
    pub e: String
}

#[derive(Deserialize, Debug)]
pub struct CachedKeys {
    keys: Vec<Key>,
    expiry: DateTime<UTC>
}

impl CachedKeys {
    pub fn new(client: &Client, discovery: &Discovery) -> Result<CachedKeys> {
        client.get(&discovery.jwks_uri)
            .send()
            .map_err(ErrorKind::Hyper)
            .and_then(|mut response| {
                let mut s = String::new();
                response.read_to_string(&mut s)
                    .map_err(ErrorKind::IO)
                    .and_then(|_| serde_json::from_str::<Keys>(&s)
                        .map_err(ErrorKind::JSON))
                    .map(|keys| CachedKeys {
                        keys: keys.keys,
                        expiry: UTC::now() +
                            Duration::seconds(response.headers.get::<header::CacheControl>()
                                .and_then(|control| control.iter()
                                    .filter_map(|cache_opt| match *cache_opt {
                                        header::CacheDirective::MaxAge(age) => Some(age as i64),
                                        _ => None
                                    }).next()
                                ).unwrap_or_else(|| 0))
                    })
            }).map_err(Error::from)
    }

    pub fn from_cache<T: Read>(read: T) -> Result<CachedKeys> {
        serde_json::from_reader(read).map_err(|err| ErrorKind::JSON(err).into())
    }

    pub fn refresh(&mut self, client: &Client, discovery: &Discovery) -> Result<()> {
        CachedKeys::new(client, discovery).map(move |keys| {
            self.keys = keys.keys;
            self.expiry = keys.expiry;
            ()
        })
    }

    pub fn keys(&mut self, client: &Client, discovery: &Discovery) -> Result<&Vec<Key>> {
        if self.is_expired() {
            CachedKeys::new(client, discovery).map(move |keys| {
                self.keys = keys.keys;
                self.expiry = keys.expiry;
                &self.keys
            })
        } else {
            Ok(&self.keys)
        }
    }

    pub fn keys_opt(&self) -> Option<&Vec<Key>> {
        if self.is_expired() {
            None
        } else {
            Some(&self.keys)
        }
    }

    pub fn is_expired(&self) -> bool {
        UTC::now() >= self.expiry
    }
}