use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use super::error::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize)]
/// Specifies if the Authorization Server prompts the End User for reauthentication and consent, as
/// defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims).
pub enum Prompt {
    /// The Authorization Server will not display any authentication or consent interfaces,
    /// returning an error if the End User is not authenticated.
    None,
    /// The Authorization Server should prompt the End-User for reauthentication.
    Login,
    /// The Authorization Server should prompt the End-User for consent.
    Consent,
    /// The Authorization Server should prompt the End-User to select an account.
    SelectAccount
}

impl AsRef<str> for Prompt {
    fn as_ref(&self) -> &str {
        match *self {
            Prompt::None => "none",
            Prompt::Login => "login",
            Prompt::Consent => "consent",
            Prompt::SelectAccount => "select_account",
        }
    }
}

impl Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for Prompt {
    type Err = Error;

    /// This is case-sensitive to [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims),
    /// returning an [Error](error/struct.error.html) wrapping a [ParsePromptError](error/enum.ErrorKind.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use ::openid_connect::request::Prompt;
    /// assert!(Prompt::from_str("JUST DO IT").is_err());
    ///
    /// assert_eq!(Prompt::None, Prompt::from_str("none").unwrap());
    /// assert_eq!(Prompt::Login, Prompt::from_str("login").unwrap());
    /// assert_eq!(Prompt::Consent, Prompt::from_str("consent").unwrap());
    /// assert_eq!(Prompt::SelectAccount, Prompt::from_str("select_account").unwrap());
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "none" => Ok(Prompt::None),
            "login" => Ok(Prompt::Login),
            "consent" => Ok(Prompt::Consent),
            "select_account" => Ok(Prompt::SelectAccount),
            _ => Err(Error::from(ErrorKind::ParsePromptError))
        }
    }
}