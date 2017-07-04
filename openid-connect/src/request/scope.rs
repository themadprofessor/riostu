use std::ops::Deref;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize)]
/// OpenID Scope Claims as defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims).
pub enum Scope {
    /// Requests access to the end user's default profile claims.
    Profile,
    /// Requests access to the email and email_verified claims.
    Email,
    /// Requests access to the address claim.
    Address,
    /// Requests access to the phone_number and phone_number_verified claims.
    Phone,
    /// Custom scope not defined by the OpenID spec, such as Google APIs.
    Ext(String)
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl Deref for Scope {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match *self {
            Scope::Profile => "profile",
            Scope::Email => "email",
            Scope::Address => "address",
            Scope::Phone => "phone",
            Scope::Ext(ref s) => s
        }
    }
}

impl FromStr for Scope {
    type Err = super::error::Error;

    /// This is case-sensitive to [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims),
    /// returning an Ext containing a string which is not defined in the spec.
    /// # Examples
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use ::openid_connect::request::Scope;
    /// assert!(Scope::from_str("Scopey McScopeface").is_ok());
    ///
    /// assert_eq!(Scope::Profile, Scope::from_str("profile").unwrap());
    /// assert_eq!(Scope::Email, Scope::from_str("email").unwrap());
    /// assert_eq!(Scope::Address, Scope::from_str("address").unwrap());
    /// assert_eq!(Scope::Phone, Scope::from_str("phone").unwrap());
    /// assert_eq!(Scope::Ext("google.scope".to_string()), Scope::from_str("google.scope").unwrap());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "profile" => Ok(Scope::Profile),
            "email" => Ok(Scope::Email),
            "address" => Ok(Scope::Address),
            "phone" => Ok(Scope::Phone),
            _ => Ok(Scope::Ext(s.to_string()))
        }
    }
}