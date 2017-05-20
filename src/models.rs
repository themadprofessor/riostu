pub struct Request {
    pub id: i32,
    pub user_id: String,
    pub amount: i32
}

pub struct NewRequest {
    pub user_id: String,
    pub amount: i32
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String
}

pub struct Token {
    pub user_id: String,
    pub token: String
}