
error_chain! {
    foreign_links {
        HyperError(::hyper::error::Error);
        JsonError(::serde_json::Error);
        IoError(::std::io::Error);
    }
}