use sea_orm::{DatabaseConnection, DbErr};

pub struct AuthService {
    conn: DatabaseConnection,
}

impl AuthService {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }
}
