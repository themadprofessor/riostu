use diesel::data_types::Cents;

use schema::requests;

#[derive(Queryable)]
pub struct Request {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub amount: Cents
}

#[derive(Insertable)]
#[table_name="requests"]
pub struct NewRequest {
    pub name: String,
    pub email: String,
    pub amount: Cents
}
