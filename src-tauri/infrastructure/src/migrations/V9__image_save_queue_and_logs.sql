-- 画像保存キューとホストログの追加

-- キュー: 保存対象の画像を記録し、非同期ワーカーが消費する
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

-- ネイティブホストの動作ログ
CREATE TABLE IF NOT EXISTS native_messaging_host_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level INTEGER NOT NULL, -- info=1, warn=2, error=3
    type INTEGER NOT NULL,  -- 例: ReceiveDmmSyncGamesRequest 等
    message TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_native_host_logs_created_at
ON native_messaging_host_logs(created_at);


