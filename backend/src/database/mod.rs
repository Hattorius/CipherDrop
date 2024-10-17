pub mod actions;
pub mod models;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;
