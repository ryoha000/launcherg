use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct SaveImageQueueTable {
    pub id: i64,
    pub src: String,
    pub src_type: i64,
    pub dst_path: String,
    pub preprocess: i64,
    pub last_error: Option<String>,
}


