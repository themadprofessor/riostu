error_chain! {
    foreign_links {
        URL(::hyper::error::ParseError);
        JSON(::serde_json::Error);
    }
}