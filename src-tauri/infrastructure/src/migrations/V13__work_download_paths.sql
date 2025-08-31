-- 作品ごとのダウンロードパス履歴
CREATE TABLE IF NOT EXISTS work_download_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    download_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_download_paths_work_id ON work_download_paths(work_id);
CREATE INDEX IF NOT EXISTS idx_work_download_paths_created_at ON work_download_paths(created_at);


