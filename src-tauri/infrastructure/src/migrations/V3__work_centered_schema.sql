-- Work中心スキーマへの統合移行（V3〜V16を統合）
-- works.id は TEXT 主キーとして、collection_elements.id を文字列化してバックフィル

-- 1) Work系の土台（idはTEXT）
CREATE TABLE IF NOT EXISTS works (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_works_title ON works(title);

-- backfill: collection_elements → works（idを文字列化）
INSERT OR IGNORE INTO works (id, title)
SELECT CAST(ce.id AS TEXT), ce.gamename FROM collection_elements ce;

-- 2) 購入作品マップ（works.id TEXT を参照）
CREATE TABLE IF NOT EXISTS dmm_works (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT NOT NULL,
    work_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dmm_works_category ON dmm_works(category);
CREATE INDEX IF NOT EXISTS idx_dmm_works_cat_sub ON dmm_works(category, subcategory);
CREATE INDEX IF NOT EXISTS idx_dmm_works_work_id ON dmm_works(work_id);

CREATE TABLE IF NOT EXISTS dlsite_works (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id TEXT NOT NULL,
    category TEXT NOT NULL,
    work_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dlsite_works_category ON dlsite_works(category);
CREATE INDEX IF NOT EXISTS idx_dlsite_works_work_id ON dlsite_works(work_id);

-- 3) 運用・ログ系
CREATE TABLE IF NOT EXISTS save_image_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    src TEXT NOT NULL,
    src_type INTEGER NOT NULL, -- 1=url, 2=path
    dst_path TEXT NOT NULL,
    preprocess INTEGER NOT NULL, -- 0=None, 1=ResizeAndCropSquare256
    last_error TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    finished_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_save_image_queue_unfinished
ON save_image_queue(finished_at)
WHERE finished_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_save_image_queue_created_at
ON save_image_queue(created_at);

CREATE TABLE IF NOT EXISTS native_messaging_host_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level INTEGER NOT NULL, -- info=1, warn=2, error=3
    type INTEGER NOT NULL,  -- 例: ReceiveDmmSyncGamesRequest 等
    message TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_native_host_logs_created_at
ON native_messaging_host_logs(created_at);

-- 4) ユーザー操作系（works.id TEXT を参照）
CREATE TABLE IF NOT EXISTS work_parent_packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    parent_pack_store_id TEXT NOT NULL,
    parent_pack_category TEXT NOT NULL,
    parent_pack_subcategory TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_parent_packs_work_id ON work_parent_packs(work_id);
CREATE INDEX IF NOT EXISTS idx_work_parent_packs_parent_key
ON work_parent_packs(parent_pack_store_id, parent_pack_category, parent_pack_subcategory);

CREATE TABLE IF NOT EXISTS work_download_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    download_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_download_paths_work_id ON work_download_paths(work_id);
CREATE INDEX IF NOT EXISTS idx_work_download_paths_created_at ON work_download_paths(created_at);

-- 5) 起動リンクとEXE保留（ce → works.id 直結でバックフィル）
CREATE TABLE IF NOT EXISTS work_lnks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    lnk_path TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_lnks_work_id ON work_lnks(work_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_work_lnks_lnk_path ON work_lnks(lnk_path);

-- backfill
INSERT OR IGNORE INTO work_lnks (work_id, lnk_path)
SELECT CAST(ce.id AS TEXT), ce.lnk_path
FROM collection_elements ce
WHERE ce.lnk_path IS NOT NULL AND ce.lnk_path <> '';

CREATE TABLE IF NOT EXISTS work_link_pending_exe (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    exe_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO work_link_pending_exe (work_id, exe_path)
SELECT CAST(ce.id AS TEXT), ce.exe_path
FROM collection_elements ce
WHERE (ce.exe_path IS NOT NULL AND ce.exe_path <> '')
  AND NOT EXISTS (SELECT 1 FROM work_lnks wl WHERE wl.work_id = CAST(ce.id AS TEXT));

-- 6) EGS関連（中間テーブルを作らず直接ワークに対応づけ）
CREATE TABLE IF NOT EXISTS work_erogamescape_map (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL UNIQUE,
    erogamescape_id INTEGER NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_egs_map_work_id ON work_erogamescape_map(work_id);
CREATE INDEX IF NOT EXISTS idx_work_egs_map_egs_id ON work_erogamescape_map(erogamescape_id);

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

-- backfill: ce.id をそのまま EGS ID として流用
INSERT OR IGNORE INTO work_erogamescape_map (work_id, erogamescape_id, created_at, updated_at)
SELECT CAST(ce.id AS TEXT), ce.id, ce.created_at, ce.updated_at
FROM collection_elements ce;

INSERT OR IGNORE INTO erogamescape_information (
    id, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby, created_at, updated_at)
SELECT ce.id, d.gamename_ruby, d.sellday, d.is_nukige, d.brandname, d.brandname_ruby,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
FROM collection_elements ce
JOIN collection_element_details d ON d.collection_element_id = ce.id;

-- 7) 活動系（V16相当を works.id TEXT に）
CREATE TABLE IF NOT EXISTS work_installs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    install_at DATETIME NOT NULL,
    original_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_installs_work_id ON work_installs(work_id);

CREATE TABLE IF NOT EXISTS work_plays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    last_play_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_plays_work_id ON work_plays(work_id);

CREATE TABLE IF NOT EXISTS work_likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    like_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_likes_work_id ON work_likes(work_id);

CREATE TABLE IF NOT EXISTS work_thumbnails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id TEXT NOT NULL,
    thumbnail_width INTEGER,
    thumbnail_height INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_thumbnails_work_id ON work_thumbnails(work_id);

-- backfill: ce → work_*（work_id を CAST）
INSERT OR IGNORE INTO work_installs (work_id, install_at, original_path, created_at, updated_at)
SELECT CAST(ce.id AS TEXT),
       ce.install_at,
       CASE
         WHEN ce.lnk_path IS NOT NULL AND ce.lnk_path <> '' THEN ce.lnk_path
         WHEN ce.exe_path IS NOT NULL AND ce.exe_path <> '' THEN ce.exe_path
         ELSE ''
       END,
       ce.created_at,
       ce.updated_at
FROM collection_elements ce WHERE ce.install_at IS NOT NULL;

INSERT OR IGNORE INTO work_plays (work_id, last_play_at, created_at, updated_at)
SELECT CAST(ce.id AS TEXT), ce.last_play_at, ce.created_at, ce.updated_at
FROM collection_elements ce WHERE ce.last_play_at IS NOT NULL;

INSERT OR IGNORE INTO work_likes (work_id, like_at, created_at, updated_at)
SELECT CAST(ce.id AS TEXT), ce.like_at, ce.created_at, ce.updated_at
FROM collection_elements ce WHERE ce.like_at IS NOT NULL;

INSERT OR IGNORE INTO work_thumbnails (work_id, thumbnail_width, thumbnail_height, created_at, updated_at)
SELECT CAST(ce.id AS TEXT), ce.thumbnail_width, ce.thumbnail_height, ce.created_at, ce.updated_at
FROM collection_elements ce
WHERE ce.thumbnail_width IS NOT NULL OR ce.thumbnail_height IS NOT NULL;

