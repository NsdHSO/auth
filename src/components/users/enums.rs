use uuid::Uuid;

pub enum SearchValue {
    Uuid(Uuid),
    String(String),
}