use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct NativeHostLogTable {
    pub id: i64,
    pub level: i64,
    pub r#type: i64,
    pub message: String,
    pub created_at: NaiveDateTime,
}


