use diesel::data_types::Cents;

use schema::{users, requests};

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
pub struct Request {
    pub id: i32,
    pub user_id: String,
    pub amount: i32
}

#[derive(Insertable)]
#[table_name="requests"]
pub struct NewRequest {
    pub user_id: String,
    pub amount: i32
}

#[derive(Insertable, Identifiable)]
#[has_many(requests)]
#[table_name="users"]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String
}

#[derive(Queryable, Identifiable)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str
}
