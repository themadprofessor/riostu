use std::io::Read;

use iron::prelude::*;
use iron::Handler;
use iron::status;
use urlencoded::{QueryMap, QueryResult, UrlEncodedBody};
use diesel::prelude::*;
use diesel;

use providers::{DatabaseProvider, LogProvider};
use models::User;
use schema::users;
use errors::*;

pub struct AuthHandler{}

impl Handler for AuthHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if let Some(log) = req.extensions.get::<LogProvider>() {
            let mut s = String::new();
            req.body.read_to_string(&mut s);
            info!(log, "Auth request: {}", s);
        }

        build_user(req.get::<UrlEncodedBody>())
            .and_then(|user| req.extensions.get::<DatabaseProvider>()
                .ok_or_else(|| Error::from(ErrorKind::MissingDatabaseConnection))
                .and_then(|con| con.get().map_err(|err| Error::from(ErrorKind::PoolTimeout(err))))
                .chain_err(|| ErrorKind::InternalServerError)
                .and_then(|con| diesel::insert(&user)
                    .into(users::table)
                    .get_result(&*con)
                    .map_err(|err| Error::from(ErrorKind::Diesel(err))).chain_err(|| ErrorKind::InternalServerError)))
            .map_err(|err| IronError {
                error: Box::new(err),
                response: Response::with((status::ImATeapot, "Teapot"))
            }).map(|_: ::models::Request| Response::with(status::Ok))
    }
}

fn build_user(map_res: QueryResult) -> Result<User> {
    map_res.map_err(|err| Error::from(ErrorKind::RequestBody(err)))
        .chain_err(|| ErrorKind::InternalServerError)
        .and_then(|mut map: QueryMap| {
            let id_res = map.remove("id")
                .ok_or_else(|| ErrorKind::MissingRequestData("id".to_string()))
                .and_then(|mut id| match id.len() {
                    1 => Ok(id.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("id".to_string(), 1))
                });
            let name_res = map.remove("name")
                .ok_or_else(|| ErrorKind::MissingRequestData("name".to_string()))
                .and_then(|mut name| match name.len() {
                    1 => Ok(name.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("name".to_string(), 1))
                });
            let email_res = map.remove("email")
                .ok_or_else(|| ErrorKind::MissingRequestData("email".to_string()))
                .and_then(|mut email| match email.len() {
                    1 => Ok(email.remove(0)),
                    _ => Err(ErrorKind::IncorrectCountRequestData("email".to_string(), 1))
                });

            id_res.and_then(
                |id| name_res.and_then(
                    |name| email_res.and_then(
                        |email| Ok(User{id, name, email}))))
            .map_err(Error::from)
            .chain_err(|| ErrorKind::BadRequest)
        })
}