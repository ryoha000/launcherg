-- リファクタリング: collection_elementsテーブルの正規化
-- collection_elementsは id, created_at, updated_at のみ保持し、
-- 他のデータは専用テーブルに分離する

-- まず、collection_elementsテーブルを一時的にリネーム
ALTER TABLE collection_elements RENAME TO collection_elements_old;

-- 新しいスキーマでcollection_elementsテーブルを作成（id, gamename, created_at, updated_at のみ）
CREATE TABLE collection_elements (
    id INTEGER PRIMARY KEY,
    gamename TEXT NOT NULL DEFAULT '',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 既存のデータからidとタイムスタンプをコピー
INSERT INTO collection_elements (id, created_at, updated_at)
SELECT id, created_at, updated_at FROM collection_elements_old;

-- スクレイピングデータ（erogamescape由来）
-- 既存のcollection_element_detailsの内容を統合
CREATE TABLE IF NOT EXISTS collection_element_info_by_erogamescape (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    gamename_ruby TEXT NOT NULL,
    sellday TEXT NOT NULL,
    is_nukige INTEGER NOT NULL,
    brandname TEXT NOT NULL,
    brandname_ruby TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- パス情報
CREATE TABLE IF NOT EXISTS collection_element_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    exe_path TEXT,
    lnk_path TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- インストール履歴
CREATE TABLE IF NOT EXISTS collection_element_installs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    install_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- プレイ履歴
CREATE TABLE IF NOT EXISTS collection_element_plays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    last_play_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- いいね履歴
CREATE TABLE IF NOT EXISTS collection_element_likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    like_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

-- サムネイル情報
CREATE TABLE IF NOT EXISTS collection_element_thumbnails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    thumbnail_width INTEGER,
    thumbnail_height INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(collection_element_id),
    FOREIGN KEY(collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);