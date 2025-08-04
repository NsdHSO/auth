use sea_orm::{DatabaseConnection, DbErr};

pub struct UsersService {
    conn: DatabaseConnection,
}

impl UsersService {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }
}
