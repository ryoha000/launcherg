-- DL版ゲーム管理機能のためのテーブル追加

-- collection_element_dl_stores テーブル
CREATE TABLE IF NOT EXISTS collection_element_dl_stores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    store_id TEXT NOT NULL,
    store_type TEXT NOT NULL, -- 'DMM', 'DLSite'  
    store_name TEXT NOT NULL,
    purchase_url TEXT NOT NULL,
    is_owned BOOLEAN NOT NULL DEFAULT FALSE,
    purchase_date DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id, store_id, store_type),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- 効率的な検索のためのインデックス
CREATE INDEX IF NOT EXISTS idx_dl_stores_collection_element_id ON collection_element_dl_stores(collection_element_id);
CREATE INDEX IF NOT EXISTS idx_dl_stores_store_id_type ON collection_element_dl_stores(store_id, store_type);
CREATE INDEX IF NOT EXISTS idx_dl_stores_is_owned ON collection_element_dl_stores(is_owned);