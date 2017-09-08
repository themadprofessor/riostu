use iron::prelude::*;
use iron::Handler;
use iron::method::Method;
use iron::status;
use urlencoded::{QueryResult, QueryMap, UrlEncodedBody};

use errors::*;
use providers::Database;
use models::NewRequest;

pub struct RequestHandler;

impl NewRequest {
    fn new(map_res: QueryResult) -> Result<NewRequest> {
        let mut map = map_res.map_err(|err| Error::from(ErrorKind::RequestDecodeError(err)))
            .chain_err(|| ErrorKind::InternalServerError)?;

        let user_id = map.remove("user_id")
            .ok_or_else(|| ErrorKind::MissingRequestDataError("user_id".to_string()))
            .and_then(|mut user_id| match user_id.len() {
                1 => Ok(user_id.remove(0)),
                _ => Err(ErrorKind::IncorrectCountRequestDataError("user_id".to_string(), 1))
            }).map_err(Error::from)
            .chain_err(|| ErrorKind::InternalServerError)?;

        let amount = map.remove("amount")
            .ok_or_else(|| ErrorKind::MissingRequestDataError("amount".to_string()))
            .and_then(|mut amount| match amount.len() {
                1 => Ok(amount.remove(0)),
                _ => Err(ErrorKind::IncorrectCountRequestDataError("amount".to_string(), 1))
            }).map_err(Error::from)
            .and_then(|cur: String| build_currency(&cur))
            .chain_err(|| ErrorKind::InternalServerError)?;

        Ok(NewRequest{
            user_id,
            amount,
        })
    }
}

impl Handler for RequestHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if req.method != Method::Post {
            return Ok(Response::with((status::MethodNotAllowed, "This endpoint only support POST!")))
        }

        NewRequest::new(req.get::<UrlEncodedBody>())
            .and_then(|request| req.extensions.get::<Database>()
                .ok_or_else(|| Error::from(ErrorKind::MissingDatabaseConnectionError))
                .and_then(|con| con.get().map_err(|err| Error::from(ErrorKind::PoolTimeoutError(err))))
                .chain_err(|| ErrorKind::InternalServerError)
                .and_then(|con| con.execute("INSERT INTO request (user_id, amount) VALUES ($1, $2);", &[&request.user_id, &request.amount])
                    .map_err(|err| Error::from(ErrorKind::PostgresError(err))).chain_err(|| ErrorKind::InternalServerError)))
            //.chain_error(|err| ErrorKind::InternalServerError)
            .map_err(|err| IronError {
                error: Box::new(Error::from(err)),
                response: Response::with((status::ImATeapot, "Teapot"))
            }).map(|_| Response::with(status::Ok))
    }
}

fn build_currency(s: &str) -> Result<i32> {
    s.parse::<f32>().map(|num| (num * 100.0) as i32).map_err(|_| Error::from(ErrorKind::AmountParseError))
}