-- ストアIDのDeny List テーブル
-- DMM/DLsite などのストア種別とIDの組で一意に管理する

CREATE TABLE IF NOT EXISTS denied_store_ids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_type INTEGER NOT NULL, -- 1=DMM, 2=DLsite（Rust側enumと対応）
    store_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_type, store_id)
);

-- 高速参照用の複合インデックス
CREATE INDEX IF NOT EXISTS idx_denied_store_ids_type_id ON denied_store_ids(store_type, store_id);


