-- work_lnks: 作品ごとの起動用ショートカットを中央管理
CREATE TABLE IF NOT EXISTS work_lnks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    lnk_path TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_lnks_work_id ON work_lnks(work_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_work_lnks_lnk_path ON work_lnks(lnk_path);


