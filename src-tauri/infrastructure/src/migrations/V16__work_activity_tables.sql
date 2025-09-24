-- Work 中心移行: 活動系テーブルの追加（installs/plays/likes/thumbnails）とバックフィル

-- 1) DDL: 新規テーブル
CREATE TABLE IF NOT EXISTS work_installs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    install_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_installs_work_id ON work_installs(work_id);

CREATE TABLE IF NOT EXISTS work_plays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    last_play_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_plays_work_id ON work_plays(work_id);

CREATE TABLE IF NOT EXISTS work_likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    like_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_likes_work_id ON work_likes(work_id);

CREATE TABLE IF NOT EXISTS work_thumbnails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    thumbnail_width INTEGER,
    thumbnail_height INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_work_thumbnails_work_id ON work_thumbnails(work_id);

-- 2) バックフィル: CE 由来データを CE-Work マッピング経由で移送
-- installs
INSERT OR IGNORE INTO work_installs (work_id, install_at, created_at, updated_at)
SELECT m.work_id, ci.install_at, ci.created_at, ci.updated_at
FROM work_collection_elements m
JOIN collection_element_installs ci ON ci.collection_element_id = m.collection_element_id;

-- plays
INSERT OR IGNORE INTO work_plays (work_id, last_play_at, created_at, updated_at)
SELECT m.work_id, cp.last_play_at, cp.created_at, cp.updated_at
FROM work_collection_elements m
JOIN collection_element_plays cp ON cp.collection_element_id = m.collection_element_id;

-- likes
INSERT OR IGNORE INTO work_likes (work_id, like_at, created_at, updated_at)
SELECT m.work_id, cl.like_at, cl.created_at, cl.updated_at
FROM work_collection_elements m
JOIN collection_element_likes cl ON cl.collection_element_id = m.collection_element_id;

-- thumbnails
INSERT OR IGNORE INTO work_thumbnails (work_id, thumbnail_width, thumbnail_height, created_at, updated_at)
SELECT m.work_id, ct.thumbnail_width, ct.thumbnail_height, ct.created_at, ct.updated_at
FROM work_collection_elements m
JOIN collection_element_thumbnails ct ON ct.collection_element_id = m.collection_element_id;


