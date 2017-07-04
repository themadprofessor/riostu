use std::ops::Deref;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use super::error::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize)]
/// Specifies how the Authorization Server should display authentication and consent UI to the End
/// Specifies the mechanism the Authorization Server will use for returning parameters from the
/// Authorization Endpoint, as defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest).
pub enum ResponseType {
    /// The Authorization Server needs to the generate an authorization code to be exchanged for an
    /// access token later.
    Code,
    /// The Authorization Server should generate an access token and return it in the redirect.
    Token
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl AsRef<str> for ResponseType {
    fn as_ref(&self) -> &str {
        match *self {
            ResponseType::Code => "code",
            ResponseType::Token => "Token"
        }
    }
}

impl FromStr for ResponseType {
    type Err = Error;

    /// This is case sensitive and the only accepted values are "code" and "token", returning an
    /// [Error](error/struct.error.html) wrapping a [ParseDisplayError](error/enum.ErrorKind.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use ::openid_connect::request::ResponseType;
    /// assert!(ResponseType::from_str("Nice response").is_err);
    ///
    /// assert_eq!(ResponseType::from_str("code").unwrap(), ResponseType::Code);
    /// assert_eq!(ResponseType::from_str("token").unwrap(), ResponseType::Token);
    /// ```
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "code" => Ok(ResponseType::Code),
            "token" => Ok(ResponseType::Token),
            _ => Err(Error::from(ErrorKind::ParseResponseTypeError))
        }
    }
}

impl Default for ResponseType {
    fn default() -> Self {
        ResponseType::Code
    }
}