use super::driver::Db;

pub const ONEPIECE_COLLECTION_ID: i32 = 1;
pub const ONEPIECE_COLLECTION_NAME: &str = "すべてのゲーム";

pub async fn migration() {
    let db = Db::new().await;
    let pool = db.0.clone();

    let sqls = get_migration_sqls();
    for sql in sqls.iter() {
        sqlx::query(sql).execute(&*pool).await.unwrap();
    }
}

pub async fn drop_all_table() -> anyhow::Result<()> {
    let tables = vec![
        "collections",
        "collection_element_maps",
        "collection_elements",
    ];
    let db = Db::new().await;
    let pool = db.0.clone();

    for table in tables {
        let sql = format!("DROP TABLE {};", table);
        sqlx::query(&sql).execute(&*pool).await?;
    }

    Ok(())
}

#[cfg(test)]
pub fn migration_sync(db: Db) {
    use tauri::async_runtime::block_on;

    let pool = db.0.clone();

    let sqls = get_migration_sqls();
    for sql in sqls.iter() {
        block_on(sqlx::query(sql).execute(&*pool)).unwrap();
    }
}

fn get_migration_sqls() -> Vec<String> {
    let collection = "
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);
    "
    .to_string();

    let collection_element = "
CREATE TABLE IF NOT EXISTS collection_elements (
    id INTEGER PRIMARY KEY,
    gamename TEXT NOT NULL,
    path TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
    "
    .to_string();

    let collection_element_maps = "
CREATE TABLE IF NOT EXISTS collection_element_maps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    collection_element_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    foreign key(collection_id) references collections(id),
    foreign key(collection_element_id) references collection_elements(id)
);
    "
    .to_string();

    let insert_onepiece_collection = format!(
        "
    INSERT OR IGNORE INTO collections(id, name) VALUES({}, \"{}\")
    ",
        ONEPIECE_COLLECTION_ID, ONEPIECE_COLLECTION_NAME
    );

    return vec![
        collection,
        collection_element,
        collection_element_maps,
        insert_onepiece_collection,
    ];
}
