use iron::prelude::*;
use iron::{BeforeMiddleware, typemap};
use slog::Logger;

impl typemap::Key for LogProvider {
    type Value = Logger;
}

pub struct LogProvider {
    log: Logger
}

impl LogProvider {
    pub fn new(log: Logger) -> LogProvider {
        LogProvider {
            log
        }
    }
}

impl BeforeMiddleware for LogProvider {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<LogProvider>(self.log.new(o!("endpoint" => format!("{}", req.url))));
        Ok(())
    }
}