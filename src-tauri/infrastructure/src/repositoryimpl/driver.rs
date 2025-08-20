use std::{path::Path, str::FromStr, sync::Arc};

use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tauri::AppHandle;

use crate::domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

#[derive(Clone)]
pub struct Db(pub(crate) Arc<Pool<Sqlite>>);

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

impl Db {
    pub async fn new(_handle: &AppHandle) -> Db {
        let db_filename = DirsSavePathResolver::default().db_file_path();
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

        log::info!("finish setup database. file: {:?}", db_filename);

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

        log::info!("finish setup database. file: {:?}", db_filename);

        Db(Arc::new(pool))
    }
}
