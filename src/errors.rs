
error_chain! {
    foreign_links {
        TLS(::hyper_native_tls::ServerError);
        HTTP(::iron::error::HttpError);
        IO(::std::io::Error);
        RequestBody(::urlencoded::UrlDecodingError);
        Currency(::currency::ParseCurrencyError);
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

        IncorrectRequestData(data: String) {
            description("Some of your request's data is incorrect!")
            display("The {} is incorrect in your request!", data)
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
    }
}