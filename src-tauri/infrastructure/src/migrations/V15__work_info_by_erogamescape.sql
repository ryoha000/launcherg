-- Work 中心移行: EGS マップと詳細テーブルの作成とデータ移行

-- 1) EGS マップ（work_id と erogamescape_id の対応）
CREATE TABLE IF NOT EXISTS work_erogamescape_map (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL UNIQUE,
    erogamescape_id INTEGER NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_egs_map_work_id ON work_erogamescape_map(work_id);
CREATE INDEX IF NOT EXISTS idx_work_egs_map_egs_id ON work_erogamescape_map(erogamescape_id);

-- 2) 詳細情報（ruby/ブランド等）を EGS 情報として独立テーブルに格納
CREATE TABLE IF NOT EXISTS erogamescape_information (
    id INTEGER PRIMARY KEY NOT NULL,
    gamename_ruby TEXT NOT NULL,
    sellday TEXT NOT NULL,
    is_nukige INTEGER NOT NULL,
    brandname TEXT NOT NULL,
    brandname_ruby TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3) 既存データを V1 由来テーブルからバックフィル（V3 正規化は不実施）
-- 3a) マップ
INSERT OR IGNORE INTO work_erogamescape_map (work_id, erogamescape_id, created_at, updated_at)
SELECT m.work_id, e.erogamescape_id, e.created_at, e.updated_at
FROM work_collection_elements m
JOIN collection_element_erogamescape_map e ON e.collection_element_id = m.collection_element_id;

-- 3b) 詳細（EGS 情報）
INSERT OR IGNORE INTO erogamescape_information (
    id, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby, created_at, updated_at
)
SELECT e.erogamescape_id, d.gamename_ruby, d.sellday, d.is_nukige, d.brandname, d.brandname_ruby,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
FROM work_collection_elements m
JOIN collection_element_erogamescape_map e ON e.collection_element_id = m.collection_element_id
JOIN collection_element_details d ON d.collection_element_id = m.collection_element_id;
