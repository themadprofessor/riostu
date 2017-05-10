use std::io::Read;

use iron::prelude::*;
use iron::Handler;
use iron::method::Method;
use iron::status;
use slog::Logger;
use urlencoded::{QueryResult, QueryMap, UrlEncodedQuery, UrlEncodedBody};
use currency::Currency;

use errors::*;

struct IoStuRequest {
    name: String,
    email: String,
    amount: Currency
}

pub struct RequestHandler {
}

impl IoStuRequest {
    fn new(map_res: QueryResult) -> Result<IoStuRequest> {
        map_res.map_err(|err| Error::from(ErrorKind::RequestBody(err)))
            .chain_err(|| ErrorKind::InternalServerError)
            .and_then(|mut map: QueryMap| {
                let name_res = map.remove("name")
                    .ok_or(ErrorKind::MissingRequestData("name".to_string()))
                    .and_then(|mut name| match name.len() {
                        1 => Ok(name.remove(0)),
                        _ => Err(ErrorKind::IncorrectCountRequestData("name".to_string(), 1))
                    });
                let email_res = map.remove("email")
                    .ok_or(ErrorKind::MissingRequestData("email".to_string()))
                    .and_then(|mut email| match email.len() {
                        1 => Ok(email.remove(0)),
                        _ => Err(ErrorKind::IncorrectCountRequestData("email".to_string(), 1))
                    });

                let amount_res = map.remove("amount")
                    .ok_or(ErrorKind::MissingRequestData("amount".to_string()))
                    .and_then(|mut amount| match amount.len() {
                        1 => Ok(amount.remove(0)),
                        _ => Err(ErrorKind::IncorrectCountRequestData("amount".to_string(), 1))
                    }).and_then(|amount| Currency::from_str(&amount).map_err(|err| ErrorKind::Currency(err)));

                name_res.and_then(|name| email_res.and_then(|email| amount_res.and_then(|amount|
                    Ok(IoStuRequest{
                        name,
                        email,
                        amount,
                    })
                ))).map_err(Error::from).chain_err(|| ErrorKind::BadRequest)
            })
    }
}

impl Handler for RequestHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if req.method != Method::Post {
            return Ok(Response::with((status::MethodNotAllowed, "This endpoint only support POST!")))
        }

        IoStuRequest::new(req.get::<UrlEncodedBody>())
            //.chain_error(|err| ErrorKind::InternalServerError)
            .map_err(|err| IronError{
                error: Box::new(Error::from(err)),
                response: Response::with((status::ImATeapot, "Teapot"))
            }).map(|_| Response::with((status::Ok)))
    }
}