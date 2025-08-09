use std::{path::Path, str::FromStr, sync::Arc};

use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tauri::AppHandle;

use crate::infrastructure::util::get_save_root_abs_dir_with_ptr_handle;

#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Sqlite>>);

const DB_FILE: &str = "launcherg_sqlite.db3";

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

impl Db {
    pub async fn new(handle: &AppHandle) -> Db {
        let root = get_save_root_abs_dir_with_ptr_handle(handle);
        let db_filename = Path::new(&root)
            .join(Path::new(DB_FILE))
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        let pool = SqlitePoolOptions::new()
            .max_connections(256)
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", db_filename))
                    .unwrap()
                    .foreign_keys(true),
            )
            .await
            .map_err(|err| format!("{}\nfile: {}", err.to_string(), db_filename))
            .unwrap();

        // migrate
        let mut conf = Config::new(ConfigDbType::Sqlite).set_db_path(&db_filename);
        embedded::migrations::runner()
            .set_abort_divergent(false)
            .run(&mut conf)
            .unwrap();

        println!("finish setup database. file: {:?}", db_filename);

        Db(Arc::new(pool))
    }

    pub async fn from_path(db_file_path: &str) -> Db {
        let db_filename = Path::new(db_file_path)
            .to_str()
            .unwrap()
            .to_string();
        let pool = SqlitePoolOptions::new()
            .max_connections(256)
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", db_filename))
                    .unwrap()
                    .foreign_keys(true),
            )
            .await
            .map_err(|err| format!("{}\nfile: {}", err.to_string(), db_filename))
            .unwrap();

        // migrate
        let mut conf = Config::new(ConfigDbType::Sqlite).set_db_path(&db_filename);
        embedded::migrations::runner()
            .set_abort_divergent(false)
            .run(&mut conf)
            .unwrap();

        println!("finish setup database. file: {:?}", db_filename);

        Db(Arc::new(pool))
    }
}
