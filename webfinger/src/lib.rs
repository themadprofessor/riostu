#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_native_tls;
extern crate serde;
extern crate serde_json;
extern crate chrono;

mod error;

use std::collections::HashMap;

use hyper::Client;
use chrono::prelude::*;

use error::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Link {
    pub rel: String,
    #[serde(rename="type")]
    pub link_type: String,
    pub herf: String,
    pub titles: Option<HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedData {
    /// The date and time when this data become invalid and should be reacquired.
    pub expire: Option<DateTime<Utc>>,
    pub subject: Option<String>,
    pub alias: Option<Vec<String>>,
    pub properties: Option<HashMap<String, String>>,
    pub links: Option<Vec<Link>>
}

#[derive(Debug, Clone)]
pub struct Discovery {
    data: CachedData
}

impl Discovery {
    /// Gets and constructs a new Discovery.
    ///
    /// Appends ".well-known/webfinger" to the given URL, performs a HTTP GET request to the URL,
    /// and parses the resulting JSON into a Discovery object.
    ///
    /// # Errors
    /// This function will return an error if the given URL is not valid.
    pub fn new<T: hyper::client::IntoUrl>(url: T, client: &Client) {
        url.into_url()
            .and_then(|u| u.join(".well-known/webfinger"))
            .map_err(ErrorKind::URL);

    }

    /// Gets the underlying discovery data without an expiry check.
    ///
    /// # Safety
    /// If the discovery has no expiry or the expiry is known to have not be reached, this method
    /// can be safely called.
    unsafe fn get_unchecked(&self) -> &CachedData {
        &self.data
    }

    /// Gets the underlying discovery data if it has not expired.
    ///
    /// Checks if the underlying discovery data has expired as defined by [is_expired](struct.Discovery.html#method.is_expired),
    /// returning a reference to said data if it hasn't expired, None otherwise.
    pub fn get(&self) -> Option<&CachedData> {
        if self.is_expired() {
            None
        } else {
            Some(&self.data)
        }
    }

    /// Checks if discovery has expired.
    ///
    /// Returns false if this discovery has expired or doesn't have an expiry, and true otherwise.
    pub fn is_expired(&self) -> bool {
        if let Some(ref expire) = self.data.expire {
            expire <= &Utc::now()
        } else {
            false
        }
    }
}