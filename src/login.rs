use std::io::Read;

use iron::prelude::*;
use iron::{status, Handler};

use providers::Log;
use errors::*;

pub struct LoginHandler {

}

impl LoginHandler {
    pub fn new() -> LoginHandler {
        LoginHandler {}
    }
}

impl Handler for LoginHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if let Some(log) = req.extensions.get::<Log>() {
            info!(log, "Request: {:?}", req);
            let mut s = String::new();
            req.body.read_to_string(&mut s);
            info!(log, "Body: {}", s);
        }

        Ok(Response::with(status::ImATeapot))
    }
}