use std::io::Read;

use hyper::{Client, header};
use serde_json;
use chrono::prelude::*;
use chrono::Duration;

use super::error::*;

#[derive(Deserialize, Debug)]
pub struct Discovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub revocation_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct CachedDiscovery {
    discovery: Discovery,
    expires: DateTime<Utc>,
}

impl CachedDiscovery {
    pub fn new(client: &Client) -> Result<CachedDiscovery> {
        client.get("https://accounts.google.com/.well-known/openid-configuration")
            .send()
            .map_err(ErrorKind::Hyper)
            .and_then(|mut response| {
                println!("Headers: {}", response.headers);
                let mut s = String::new();
                response.read_to_string(&mut s)
                    .map_err(ErrorKind::IO)
                    .and_then(|_| serde_json::from_str::<Discovery>(&s)
                        .map_err(ErrorKind::JSON)
                        .map(|discovery| CachedDiscovery {
                            discovery,
                            expires: Utc::now() +
                                Duration::seconds(response.headers.get::<header::CacheControl>()
                                    .and_then(|control| control.iter()
                                        .filter_map(|cache_opt| match *cache_opt {
                                            header::CacheDirective::MaxAge(age) => Some(i64::from(age)),
                                            _ => None
                                        }).next()
                                    ).unwrap_or_else(|| 0))
                        }))
            }).map_err(Error::from)
    }

    pub fn from_cache<T: Read>(read: T) -> Result<CachedDiscovery> {
        serde_json::from_reader(read).map_err(|err| ErrorKind::JSON(err).into())
    }

    pub fn discovery(&mut self, client: &Client) -> Result<&Discovery> {
        if self.is_expired() {
            CachedDiscovery::new(client).map(move |disc| {
                self.discovery = disc.discovery;
                self.expires = disc.expires;
                &self.discovery
            })
        } else {
            Ok(&self.discovery)
        }
    }

    pub fn discovery_opt(&self) -> Option<&Discovery> {
        if self.is_expired() {
            None
        } else {
            Some(&self.discovery)
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires
    }
}