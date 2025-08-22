-- DMM のパック指定を保持するテーブル
-- 独立した概念のため deny list とは別テーブルとする

CREATE TABLE IF NOT EXISTS dmm_pack_marks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_id TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dmm_pack_marks_store_id
    ON dmm_pack_marks (store_id);

-- 既存テーブルとの互換性維持のため、トリガーで updated_at を自動更新
CREATE TRIGGER IF NOT EXISTS trg_dmm_pack_marks_updated
AFTER UPDATE ON dmm_pack_marks
FOR EACH ROW
BEGIN
    UPDATE dmm_pack_marks SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;


