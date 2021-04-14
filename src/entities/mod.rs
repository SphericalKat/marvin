pub struct User {
    pub user_id: i64,
    pub full_name: String,
    pub user_name: Option<String>,
}

pub struct Chat {
    pub chat_id: i64,
    pub chat_name: Option<String>,
}
