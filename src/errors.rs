
error_chain! {
    foreign_links {
        TlsError(::hyper_native_tls::ServerError);
        HttpError(::iron::error::HttpError);
        IoError(::std::io::Error);
        RequestDecodeError(::urlencoded::UrlDecodingError);
        RequestBody2Error(::bodyparser::BodyError);
        PoolInitialisationError(::r2d2::InitializationError);
        PoolTimeoutError(::r2d2::GetTimeout);
        PostgresError(::postgres::error::Error);
        GoogleError(::google::error::Error);
        Base64Error(::base64::DecodeError);
        JwtError(::jwt::errors::Error);
        JsonError(::serde_json::Error);
        ConfigError(::config::ConfigError);
        ClientTlsError(::hyper_native_tls::native_tls::Error);
    }

    errors {
        MissingConfigValueError(id: String) {
            description("Value not found in config!")
            display("The value {} was not found in the config!", id)
        }

        MissingConfigValueTableError(id: String, table: String) {
            description("Value not found in table in config!")
            display("The value {} was not found in the {} table in the config!", id, table)
        }

        InvalidConfigTypeError(id: String, t: String) {
            description("Value is not the correct type in config!")
            display("The value {} should be a {} in the config!", id, t)
        }

        LoggerConfigError(err: String) {
            description("Failed to load logger config!")
            display("The logger config couldn't be loaded! {}", err)
        }

        MissingRequestDataError(data: String) {
            description("Your request is missing some data!")
            display("The {} data is missing from your request!", data)
        }

        IncorrectCountRequestDataError(data: String, count: usize) {
            description("Your request has an incorrect amount of some data!")
            display("There can on be {} of the {} data!", count, data)
        }

        MissingRequestError {
            description("No request data given!")
        }

        InternalServerError {
            description("Server produced an error!")
        }

        BadRequestError {
            description("Your request was invalid!")
        }

        AmountParseError {
            description("Invalid amount given!")
        }

        MissingDatabaseConnectionError {
            description("No connection to the database found!")
        }

        MissingKidGoogleTokenError {
            description("No kid entry in Google's JWT header!")
        }

        NoValidKeyGoogleError {
            description("No matching key for Google's JWT!")
        }

        PoisonError(msg: String, obj: String) {
            description("Read Write Lock was poisoned!")
            display("The Read Write Lock for {} was poisoned! {}", obj, msg)
        }

        IdentityFileNotExistError(path: String) {
            description("SSL identity file doesn't exist")
            display("SSL identity file doesn't exist: {}", path)
        }
    }
}