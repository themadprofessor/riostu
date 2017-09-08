use iron::prelude::*;
use iron::{BeforeMiddleware, typemap};
use slog::Logger;

impl typemap::Key for Log {
    type Value = Logger;
}

pub struct Log {
    log: Logger
}

impl Log {
    pub fn new(log: Logger) -> Log {
        Log {
            log
        }
    }
}

impl BeforeMiddleware for Log {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Log>(self.log.new(o!("endpoint" => format!("{}", req.url))));
        Ok(())
    }
}