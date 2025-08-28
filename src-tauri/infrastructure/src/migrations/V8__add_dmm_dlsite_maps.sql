
-- 共有 Works テーブル（タイトル単位）
CREATE TABLE IF NOT EXISTS works (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_works_title ON works(title);

-- DMM の購入作品
CREATE TABLE IF NOT EXISTS dmm_works (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT NOT NULL,
    work_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dmm_works_category ON dmm_works(category);
CREATE INDEX IF NOT EXISTS idx_dmm_works_cat_sub ON dmm_works(category, subcategory);
CREATE INDEX IF NOT EXISTS idx_dmm_works_work_id ON dmm_works(work_id);

-- DLsite の購入作品
CREATE TABLE IF NOT EXISTS dlsite_works (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    work_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dlsite_works_category ON dlsite_works(category);
CREATE INDEX IF NOT EXISTS idx_dlsite_works_work_id ON dlsite_works(work_id);

-- 作品と collection_elements の関連（当面1:1想定だがN:N拡張可能）
-- 作品と collection_elements の関連（共通テーブルに統合）
CREATE TABLE IF NOT EXISTS work_collection_elements (
    work_id INTEGER NOT NULL,
    collection_element_id INTEGER NOT NULL,
    PRIMARY KEY (work_id, collection_element_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE,
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_work_single_map ON work_collection_elements(work_id);

