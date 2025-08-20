-- DMM / DLsite マッピングの追加

-- DMM マッピング
CREATE TABLE IF NOT EXISTS collection_element_dmm (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- DLsite マッピング
CREATE TABLE IF NOT EXISTS collection_element_dlsite (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_dmm_collection_element_id ON collection_element_dmm(collection_element_id);
CREATE INDEX IF NOT EXISTS idx_dmm_store_id ON collection_element_dmm(store_id);
CREATE INDEX IF NOT EXISTS idx_dlsite_collection_element_id ON collection_element_dlsite(collection_element_id);
CREATE INDEX IF NOT EXISTS idx_dlsite_store_id ON collection_element_dlsite(store_id);


