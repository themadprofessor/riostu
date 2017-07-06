use std::collections::HashSet;
use std::fmt;

use hyper::Url;
use hyper::client::IntoUrl;
use rand::{self, Rng};
use base64;
use serde_urlencoded;
use itertools::Itertools;

mod scope;
mod display;
mod prompt;
mod response_type;
pub mod error;

pub use self::scope::Scope;
pub use self::display::Display;
pub use self::prompt::Prompt;
pub use self::response_type::ResponseType;
use self::error::*;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct AuthRequest {
    scope: HashSet<Scope>,
    response_type: ResponseType,
    client_id: String,
    #[serde(serialize_with = "ser_url")]
    redirect_uri: Url,
    state: String,
    nonce: Option<String>,
    prompt: Option<Prompt>,
    max_age: Option<usize>,
    ui_locales: Option<Vec<String>>,
    id_token_hint: Option<String>,
    login_hint: Option<String>,
    acr_values: Option<Vec<String>>
}

impl AuthRequest {
    pub fn new<A, B, C, S, T, U, V, W>(scopes: C, client_id: T, redirect_uri: V, response_type: U, state: W) -> Result<AuthRequest> where
        S: Into<ResponseType>,
        T: Into<String>,
        U: Into<Option<S>>,
        V: IntoUrl,
        W: Into<Option<String>>,
        A: Into<Scope>,
        B: Iterator<Item=A>,
        C: IntoIterator<Item=A, IntoIter=B> {

        let uri = redirect_uri.into_url().map_err(|err| Error::from(ErrorKind::URL(err)))?;
        let tmp_state = match state.into() {
            Some(s) => Ok(s),
            None => gen_hash()
        }.map_err(Error::from)?;

        Ok(AuthRequest {
            scope: scopes.into_iter().map(Into::into).collect::<HashSet<Scope>>(),
            response_type: response_type.into().map(Into::into).unwrap_or_default(),
            client_id: client_id.into(),
            redirect_uri: uri,
            state: tmp_state,
            nonce: None,
            prompt: None,
            max_age: None,
            ui_locales: None,
            id_token_hint: None,
            login_hint: None,
            acr_values: None

        })
    }

    pub fn add_scope<T: Into<Scope>>(&mut self, scope: Scope) -> &mut Self {
        self.scope.insert(scope);
        self
    }

    pub fn set_scopes<I, T, S> (&mut self, scopes: T) -> &mut Self where S: Into<Scope>,
                                                                  I: Iterator<Item=S>,
                                                                  T: IntoIterator<Item=S, IntoIter=I> {
        self.scope.extend(scopes.into_iter().map(Into::into));
        self
    }

    pub fn set_response_type<T>(&mut self, response: T) -> &mut Self where T: Into<ResponseType> {
        self.response_type = response.into();
        self
    }

    pub fn set_client_id<T>(&mut self, id: T) -> &mut Self where T: Into<String> {
        self.client_id = id.into();
        self
    }

    pub fn set_redirect_uri<T>(&mut self, uri: T) -> Result<&mut Self> where T: IntoUrl {
        self.redirect_uri = uri.into_url().map_err(|err| Error::from(ErrorKind::URL(err)))?;
        Ok(self)
    }

    pub fn set_state<T>(&mut self, state: T) -> &mut Self where T: Into<String> {
        self.state = state.into();
        self
    }

    pub fn set_nonce<T>(&mut self, nonce: T) -> &mut Self where T: Into<String> {
        self.nonce = Some(nonce.into());
        self
    }

    pub fn set_prompt<T>(&mut self, prompt: T) -> &mut Self where T: Into<Prompt> {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn set_max_age<T>(&mut self, max_age: T) -> &mut Self where T: Into<usize> {
        self.max_age = Some(max_age.into());
        self
    }

    pub fn add_ui_locale<T>(&mut self, locale: T) -> &mut Self where T: Into<String> {
        match self.ui_locales {
            Some(ref mut locales) => locales.push(locale.into()),
            None => self.ui_locales = Some(vec![locale.into()])
        };
        self
    }

    pub fn set_ui_locales<T, S, U>(&mut self, locales: T) -> &mut Self where S: Into<String>,
                                                                         U: Iterator<Item=S>,
                                                                         T: IntoIterator<Item=S, IntoIter=U> {
        match self.ui_locales {
            Some(ref mut locs) => locs.extend(locales.into_iter().map(Into::into)),
            None => self.ui_locales = Some(locales.into_iter()
                .map(Into::into)
                .collect::<Vec<String>>())
        };
        self
    }

    pub fn set_id_token_hint<T>(&mut self, hint: T) -> &mut Self where T: Into<String> {
        self.id_token_hint = Some(hint.into());
        self
    }

    pub fn set_login_hint<T>(&mut self, hint: T) -> &mut Self where T: Into<String> {
        self.login_hint = Some(hint.into());
        self
    }

    pub fn add_acr_value<T>(&mut self, value: T) -> &mut Self where T: Into<String> {
        match self.acr_values {
            Some(ref mut vals) => vals.push(value.into()),
            None => self.acr_values = Some(vec![value.into()])
        };
        self
    }

    pub fn set_acr_values<T, S, U>(&mut self, values: T) -> &mut Self where S: Into<String>,
                                                                     U: Iterator<Item=S>,
                                                                     T: IntoIterator<Item=S, IntoIter=U> {
        match self.acr_values {
            Some(ref mut vals) => vals.extend(values.into_iter().map(Into::into)),
            None => self.acr_values = Some(values.into_iter()
                .map(Into::into)
                .collect::<Vec<String>>())
        };
        self
    }

    pub fn scopes(&self) -> &HashSet<Scope> {
        &self.scope
    }

    pub fn response_type(&self) -> &ResponseType {
        &self.response_type
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn redirect_uri(&self) -> &Url {
        &self.redirect_uri
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn nonce(&self) -> Option<&str> {
        self.nonce.as_ref().map(String::as_ref)
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        self.prompt.as_ref()
    }

    pub fn max_age(&self) -> Option<&usize> {
        self.max_age.as_ref()
    }

    pub fn ui_locales(&self) -> Option<&[String]> {
        self.ui_locales.as_ref().map(Vec::as_ref)
    }

    pub fn id_token_hint(&self) -> Option<&str> {
        self.id_token_hint.as_ref().map(String::as_ref)
    }

    pub fn acr_values(&self) -> Option<&[String]> {
        self.acr_values.as_ref().map(Vec::as_ref)
    }

    pub fn to_url<T>(&self, base: T) -> Result<Url> where T: IntoUrl {
        let mut url = base.into_url().map_err(|err| Error::from(ErrorKind::URL(err)))?;
        {
            let mut pairs = url.query_pairs_mut();

            pairs.append_pair("response_type", self.response_type.as_ref())
                .append_pair("scope", self.scope.iter()
                    .map(AsRef::as_ref)
                    .chain(::std::iter::once("openid"))
                    .join("%20")
                    .as_ref())
                .append_pair("client_id", self.client_id.as_ref())
                .append_pair("redirect_uri", self.redirect_uri.as_ref())
                .append_pair("state", encode(&self.state).as_ref());
            if let Some(nonce) = self.nonce.as_ref() {
                pairs.append_pair("nonce", encode(nonce).as_ref());
            }
            if let Some(prompt) = self.prompt.as_ref() {
                pairs.append_pair("prompt", prompt.as_ref());
            }
            if let Some(max_age) = self.max_age.as_ref() {
                pairs.append_pair("max_age", max_age.to_string().as_ref());
            }
            if let Some(ui_locales) = self.ui_locales.as_ref() {
                let s = ui_locales.iter().join("%20");
                pairs.append_pair("ui_locales", encode(&s).as_ref());
            }
            if let Some(id_token_hint) = self.id_token_hint.as_ref() {
                pairs.append_pair("id_token_hint", encode(id_token_hint).as_ref());
            }
            if let Some(login_hint) = self.login_hint.as_ref() {
                pairs.append_pair("login_hint", encode(login_hint).as_ref());
            }
            if let Some(acr_values) = self.acr_values.as_ref() {
                let s = acr_values.iter().join("%20");
                pairs.append_pair("acr_values", encode(&s).as_ref());
            }
        }

        Ok(url)
    }
}

impl fmt::Display for AuthRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).expect("AuthRequest failed url encoding");
        write!(f, "{}", s)
    }
}

#[inline]
fn gen_hash() -> ::std::result::Result<String, ErrorKind> {
    rand::OsRng::new().map(|mut rng| {
        base64::encode_config(&rng.gen_iter::<u8>().take(30).collect::<Vec<u8>>(), base64::URL_SAFE)
    }).map_err(ErrorKind::IO)
}

#[inline]
fn ser_url<T>(me: &Url, ser: T) -> ::std::result::Result<T::Ok, T::Error> where T: ::serde::Serializer {
    ser.serialize_str(me.as_str())
}

#[inline]
fn encode<T>(data: &T) -> String where T: AsRef<[u8]> {
    base64::encode_config(data, base64::URL_SAFE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        println!("{}", AuthRequest::new(vec![Scope::Profile],
                                        "id".to_string(),
                                        "http://127.0.0.1",
                                        ResponseType::Code,
                                        None).unwrap().to_url("http://127.0.0.1").unwrap());
    }
}