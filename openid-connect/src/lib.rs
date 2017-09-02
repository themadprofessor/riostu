#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_native_tls;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate rand;
extern crate base64;
extern crate itertools;

pub mod request;
pub mod response;

#[cfg(test)]
mod tests {
    use super::*;
    use request::{Scope, ResponseType};

    #[test]
    fn test() {
        /*
        https://accounts.google.com/o/oauth2/v2/auth
        */
        let request::AuthRequest::new(Scope::Profile,
                                      "748001161761-8h45hco16bd6sjgbla3m1qk5pdutu0cu.apps.googleusercontent.com");
    }
}