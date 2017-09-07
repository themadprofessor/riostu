
error_chain! {
    foreign_links {
        TLS(::hyper_native_tls::ServerError);
        HTTP(::iron::error::HttpError);
        IO(::std::io::Error);
        RequestBody(::urlencoded::UrlDecodingError);
        RequestBody2(::bodyparser::BodyError);
        PoolInitialisation(::r2d2::InitializationError);
        PoolTimeout(::r2d2::GetTimeout);
        Postgres(::postgres::error::Error);
        Google(::google::error::Error);
        Base64(::base64::DecodeError);
        JWT(::jwt::errors::Error);
        JSON(::serde_json::Error);
        Config(::config::ConfigError);
        ClientTlsError(::hyper_native_tls::native_tls::Error);
    }

    errors {
        MissingConfigValue(id: String) {
            description("Value not found in config!")
            display("The value {} was not found in the config!", id)
        }

        MissingConfigValueTable(id: String, table: String) {
            description("Value not found in table in config!")
            display("The value {} was not found in the {} table in the config!", id, table)
        }

        InvalidConfigType(id: String, t: String) {
            description("Value is not the correct type in config!")
            display("The value {} should be a {} in the config!", id, t)
        }

        LoggerConfig(err: String) {
            description("Failed to load logger config!")
            display("The logger config couldn't be loaded! {}", err)
        }

        MissingRequestData(data: String) {
            description("Your request is missing some data!")
            display("The {} data is missing from your request!", data)
        }

        IncorrectCountRequestData(data: String, count: usize) {
            description("Your request has an incorrect amount of some data!")
            display("There can on be {} of the {} data!", count, data)
        }

        MissingRequest {
            description("No request data given!")
        }

        InternalServerError {
            description("Server produced an error!")
        }

        BadRequest {
            description("Your request was invalid!")
        }

        AmountParse {
            description("Invalid amount given!")
        }

        MissingDatabaseConnection {
            description("No connection to the database found!")
        }

        MissingKidGoogleToken {
            description("No kid entry in Google's JWT header!")
        }

        NoValidKeyGoogle {
            description("No matching key for Google's JWT!")
        }

        Poison(msg: String, obj: String) {
            description("Read Write Lock was poisoned!")
            display("The Read Write Lock for {} was poisoned! {}", obj, msg)
        }
    }
}