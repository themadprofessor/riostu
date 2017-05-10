use std::ops::Deref;

use iron::prelude::*;
use iron::{AfterMiddleware, status};
use slog::Logger;

use logging::LoggerMiddleware;
use errors::*;

pub struct ErrorCapture {
}

impl AfterMiddleware for ErrorCapture {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        let info = format!("{}", err);
        if let Some(log) = req.extensions.get::<LoggerMiddleware>() {
            warn!(log, "Error produced by {} endpoint! {}", req.url, info);
        }
        Ok(match err.error.deref().downcast::<::errors::Error>().map(|e| e.deref()) {
            Some(error) => match error {
                &ErrorKind::BadRequest => Response::with((status::BadRequest, info)),
                _ => Response::with((status::InternalServerError, info))
            },
            None => {
                Response::with((status::InternalServerError, info))
            }
        })
    }
}