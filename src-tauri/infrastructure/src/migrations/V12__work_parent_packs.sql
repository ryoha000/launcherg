-- work_parent_packs: 親パックと子作品の対応
CREATE TABLE IF NOT EXISTS work_parent_packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_id INTEGER NOT NULL,
    parent_pack_work_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(work_id, parent_pack_work_id),
    FOREIGN KEY(work_id) REFERENCES works(id) ON DELETE CASCADE,
    FOREIGN KEY(parent_pack_work_id) REFERENCES works(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_parent_packs_work_id ON work_parent_packs(work_id);
CREATE INDEX IF NOT EXISTS idx_work_parent_packs_parent_id ON work_parent_packs(parent_pack_work_id);


