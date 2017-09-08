use std::ops::Deref;
use std::error::Error;

use iron::prelude::*;
use iron::{AfterMiddleware, status};

use providers::Log;
use errors::*;

pub struct ErrorCapture {
}

impl AfterMiddleware for ErrorCapture {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        let info = err.error.cause().map(|e| format!("{}", e)).unwrap_or_else(|| format!("{}", err.error));
        if let Some(log) = req.extensions.get::<Log>() {
            warn!(log, "Error during handling"; "url" => req.url.as_ref().as_str(), "desc" => err.description());
            trace!(log, "{:?}", err)
        }
        Ok(match err.error.deref().downcast::<::errors::Error>().map(|e| e.deref()) {
            Some(error) => match *error {
                ErrorKind::BadRequestError => Response::with((status::BadRequest, info)),
                _ => Response::with((status::InternalServerError, info))
            },
            None => {
                Response::with((status::InternalServerError, info))
            }
        })
    }
}