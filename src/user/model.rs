#[derive(Debug)]
pub struct UserContext {
    pub id: i64,
    pub is_user: bool,
    pub username: String,
}

pub struct IdPassword {
    pub id: i64,
    pub password: Box<[u8]>,
}

pub struct IdUsername {
    pub id: i64,
    pub username: String,
}
