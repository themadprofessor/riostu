use iron::prelude::*;
use iron::Handler;
use iron::method::Method;
use iron::status;
use urlencoded::{QueryResult, QueryMap, UrlEncodedBody};
use diesel::pg::data_types::Cents;
use diesel::prelude::*;
use diesel;

use errors::*;
use models::NewRequest;
use schema::requests;
use providers::DatabaseProvider;

pub struct RequestHandler;

impl NewRequest {
    fn new(map_res: QueryResult) -> Result<NewRequest> {
        map_res.map_err(|err| Error::from(ErrorKind::RequestBody(err)))
            .chain_err(|| ErrorKind::InternalServerError)
            .and_then(|mut map: QueryMap| {
                let user_id_res = map.remove("user_id")
                    .ok_or_else(|| ErrorKind::MissingRequestData("user_id".to_string()))
                    .and_then(|mut user_id| match user_id.len() {
                        1 => Ok(user_id.remove(0)),
                        _ => Err(ErrorKind::IncorrectCountRequestData("user_id".to_string(), 1))
                    }).map_err(Error::from);

                let amount_res = map.remove("amount")
                    .ok_or_else(|| ErrorKind::MissingRequestData("amount".to_string()))
                    .and_then(|mut amount| match amount.len() {
                        1 => Ok(amount.remove(0)),
                        _ => Err(ErrorKind::IncorrectCountRequestData("amount".to_string(), 1))
                    }).map_err(Error::from).and_then(|amount| build_currency(amount));

                user_id_res.and_then(|user_id| amount_res.and_then(|amount|
                    Ok(NewRequest{
                        user_id,
                        amount,
                    })
                )).map_err(Error::from).chain_err(|| ErrorKind::BadRequest)
            })
    }
}

impl Handler for RequestHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if req.method != Method::Post {
            return Ok(Response::with((status::MethodNotAllowed, "This endpoint only support POST!")))
        }

        NewRequest::new(req.get::<UrlEncodedBody>())
            .and_then(|request| req.extensions.get::<DatabaseProvider>()
                .ok_or_else(|| Error::from(ErrorKind::MissingDatabaseConnection))
                .and_then(|con| con.get().map_err(|err| Error::from(ErrorKind::PoolTimeout(err))))
                .chain_err(|| ErrorKind::InternalServerError)
                .and_then(|con| diesel::insert(&request)
                    .into(requests::table)
                    .get_result(&*con)
                    .map_err(|err| Error::from(ErrorKind::Diesel(err))).chain_err(|| ErrorKind::InternalServerError)))
            //.chain_error(|err| ErrorKind::InternalServerError)
            .map_err(|err| IronError {
                error: Box::new(Error::from(err)),
                response: Response::with((status::ImATeapot, "Teapot"))
            }).map(|_: ::models::Request| Response::with(status::Ok))
    }
}

fn build_currency(s: String) -> Result<i32> {
    s.parse::<f32>().map(|num| (num * 100 as f32) as i32).map_err(|err| Error::from(ErrorKind::AmountParse))
}