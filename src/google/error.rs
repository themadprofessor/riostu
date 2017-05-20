
error_chain! {
    foreign_links {
        Hyper(::hyper::error::Error);
        JSON(::serde_json::Error);
        IO(::std::io::Error);
    }

    errors {
        UnexpectedGoogleStatus(code: ::hyper::status::StatusCode, info: String) {
            description("Received an unexpected HTTP status code from Google!")
            display("Received a {} from Google. {}", code, info)
        }

        ExpiredDiscovery {
            description("The cached version of discovery has expired!")
        }

        ExpiredKeys {
            description("The cached version of keys has expired!")
        }
    }
}