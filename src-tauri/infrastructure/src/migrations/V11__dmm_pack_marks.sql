-- DMM 作品に対するパック関連付け（作品側にぶら下げる）

CREATE TABLE IF NOT EXISTS dmm_work_packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dmm_work_packs_work_id ON dmm_work_packs(work_id);

