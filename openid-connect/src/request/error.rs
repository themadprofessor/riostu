//! The errors which can occur while creating a request.

error_chain! {
    foreign_links {
        IO(::std::io::Error);
        URL(::hyper::error::ParseError);
    }

    errors {
        /// Failed to parse a scope value as defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#ScopeClaims).
        ParseScopeError {
            description("Failed to parse Scope value!")
        }

        /// Failed to parse a display value as defined by [the spec](https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest).
        ParseDisplayError {
            description("Failed to parse Display value!")
        }

        /// Failed to parse a prompt value as defined by [the spec](http://openid.net/specs/openid-connect-core-1_0.html#AuthRequest).
        ParsePromptError {
            description("Failed to parse Prompt value!")
        }

        /// Failed to parse a response type value as defined by [the spec](http://openid.net/specs/openid-connect-core-1_0.html#AuthRequest)
        ParseResponseTypeError {
            description("Failed to parse Response Type value!")
        }
    }
}