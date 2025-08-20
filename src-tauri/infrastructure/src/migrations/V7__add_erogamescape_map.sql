-- EGS連携マップテーブルの追加とバックフィル

CREATE TABLE IF NOT EXISTS collection_element_erogamescape_map (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    erogamescape_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    UNIQUE(erogamescape_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- 既存データのバックフィル（id=EGS ID として運用していたもの）
INSERT OR IGNORE INTO collection_element_erogamescape_map (collection_element_id, erogamescape_id)
SELECT id, id FROM collection_elements;

-- 効率的な検索のためのインデックス
CREATE INDEX IF NOT EXISTS idx_egs_map_collection_element_id ON collection_element_erogamescape_map(collection_element_id);
CREATE INDEX IF NOT EXISTS idx_egs_map_erogamescape_id ON collection_element_erogamescape_map(erogamescape_id);


