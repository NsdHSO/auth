use sea_orm::DatabaseConnection;

#[allow(dead_code)]
pub struct UsersService {
    #[allow(dead_code)]
    conn: DatabaseConnection,
}

impl UsersService {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }
}
