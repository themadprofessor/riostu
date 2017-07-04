use std::ops::Deref;
use std::fmt;
use std::str::FromStr;
use std::default::Default;

use super::error::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize)]
/// Specifies how the Authorization Server should display authentication and consent UI to the End
/// User, as defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest).
pub enum Display {
    /// The Authorization Server should use a full User Agent page view. This is the default value.
    Page,
    /// The Authorization Server should use a popup User Agent window.
    Popup,
    /// The Authorization Server should use an interface for a device which utilises a touch interface.
    Touch,
    /// The Authorization Server should use an interface for a feature phone type display.
    Wap
}

impl Deref for Display {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match *self {
            Display::Page => "page",
            Display::Popup => "popup",
            Display::Touch => "touch",
            Display::Wap => "wap",
        }
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl FromStr for Display {
    type Err = Error;

    /// This is case-sensitive to [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims),
    /// returning an [Error](error/struct.error.html) wrapping a [ParseDisplayError](error/enum.ErrorKind.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use ::openid_connect::request::Display;
    /// assert!(Display::from_str("ULTRA WIDE").is_err());
    ///
    /// assert_eq!(Display::Page, Display::from_str("page").unwrap());
    /// assert_eq!(Display::Popup, Display::from_str("popup").unwrap());
    /// assert_eq!(Display::Touch, Display::from_str("touch").unwrap());
    /// assert_eq!(Display::Wap, Display::from_str("wap").unwrap());
    /// ```
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "page" => Ok(Display::Page),
            "popup" => Ok(Display::Popup),
            "touch" => Ok(Display::Touch),
            "wap" => Ok(Display::Wap),
            _ => Err(Error::from(ErrorKind::ParseDisplayError))
        }
    }
}

impl Default for Display {
    /// Return the Page value.
    fn default() -> Self {
        Display::Page
    }
}