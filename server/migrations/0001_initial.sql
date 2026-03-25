CREATE TABLE IF NOT EXISTS devices (
  device_id TEXT PRIMARY KEY,
  secret_hash TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_synced_at TEXT
);

CREATE TABLE IF NOT EXISTS device_work_snapshots (
  device_id TEXT NOT NULL,
  work_id TEXT NOT NULL,
  title TEXT NOT NULL,
  image_key TEXT,
  thumbnail_width INTEGER,
  thumbnail_height INTEGER,
  original_path TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (device_id, work_id),
  FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_device_work_snapshots_device_id
ON device_work_snapshots (device_id);
