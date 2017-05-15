use std::ops::Deref;

use iron::prelude::*;
use iron::{AfterMiddleware, status};

use providers::LogProvider;
use errors::*;

pub struct ErrorCapture {
}

impl AfterMiddleware for ErrorCapture {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        let info = err.error.cause().map(|e| format!("{}", e)).unwrap_or_else(|| format!("{}", err.error));
        if let Some(log) = req.extensions.get::<LogProvider>() {
            warn!(log, "Error produced by {} endpoint! {}", req.url, info);
            trace!(log, "{:?}", err)
        }
        Ok(match err.error.deref().downcast::<::errors::Error>().map(|e| e.deref()) {
            Some(error) => match *error {
                ErrorKind::BadRequest => Response::with((status::BadRequest, info)),
                _ => Response::with((status::InternalServerError, info))
            },
            None => {
                Response::with((status::InternalServerError, info))
            }
        })
    }
}