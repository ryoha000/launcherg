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

-- Backfill: collection_elements.lnk_path -> work_lnks
INSERT OR IGNORE INTO work_lnks (work_id, lnk_path)
SELECT m.work_id, ce.lnk_path
FROM work_collection_elements m
JOIN collection_elements ce ON ce.id = m.collection_element_id
WHERE ce.lnk_path IS NOT NULL AND ce.lnk_path <> '';

-- Pending exe records for later .lnk generation
CREATE TABLE IF NOT EXISTS work_link_pending_exe (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    exe_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO work_link_pending_exe (work_id, exe_path)
SELECT m.work_id, ce.exe_path
FROM work_collection_elements m
JOIN collection_elements ce ON ce.id = m.collection_element_id
WHERE (ce.exe_path IS NOT NULL AND ce.exe_path <> '')
  AND NOT EXISTS (
    SELECT 1 FROM work_lnks wl WHERE wl.work_id = m.work_id
  );
