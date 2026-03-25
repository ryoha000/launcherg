ALTER TABLE device_work_snapshots ADD COLUMN erogamescape_id INTEGER;

CREATE TABLE IF NOT EXISTS remote_share_images (
  dedupe_key TEXT PRIMARY KEY,
  image_key TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_device_work_snapshots_device_id_erogamescape_id
ON device_work_snapshots (device_id, erogamescape_id);
