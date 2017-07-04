
error_chain! {
    foreign_links {
        Hyper(::hyper::error::Error);
        JSON(::serde_json::Error);
        IO(::std::io::Error);
    }
}