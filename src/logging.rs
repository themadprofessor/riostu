use iron::prelude::*;
use iron::{BeforeMiddleware, typemap};
use slog::Logger;

impl typemap::Key for LoggerMiddleware {
    type Value = Logger;
}

pub struct LoggerMiddleware {
    log: Logger
}

impl LoggerMiddleware {
    pub fn new(log: Logger) -> LoggerMiddleware {
        LoggerMiddleware {
            log
        }
    }
}

impl BeforeMiddleware for LoggerMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<LoggerMiddleware>(self.log.new(o!("endpoint" => format!("{}", req.url))));
        Ok(())
    }
}