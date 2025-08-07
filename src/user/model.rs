#[derive(Debug)]
pub struct UserContext {
    pub is_user: bool,
    pub username: String,
}

pub struct IdPassword {
    pub id: i64,
    pub password: String,
}

pub struct IdUsername {
    pub id: i64,
    pub username: String,
}
