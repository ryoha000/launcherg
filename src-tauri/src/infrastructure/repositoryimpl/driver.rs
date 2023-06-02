use std::{path::Path, sync::Arc};

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::infrastructure::util::get_save_root_abs_dir;

#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Sqlite>>);

const DB_FILE: &str = "launcherg_sqlite.db3";

impl Db {
    pub async fn new() -> Db {
        let root = get_save_root_abs_dir();
        let db_filename = Path::new(&root)
            .join(Path::new(DB_FILE))
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        let pool = SqlitePoolOptions::new()
            .max_connections(256)
            .connect(&format!("sqlite://{}?mode=rwc", db_filename))
            .await
            .map_err(|err| format!("{}\nfile: {}", err.to_string(), db_filename))
            .unwrap();

        println!("finish setup database. file: {:?}", db_filename);

        Db(Arc::new(pool))
    }
}
