CREATE TABLE `device_work_snapshots` (
	`device_id` text NOT NULL,
	`work_id` text NOT NULL,
	`title` text NOT NULL,
	`image_key` text,
	`thumbnail_width` integer,
	`thumbnail_height` integer,
	`original_path` text,
	`erogamescape_id` integer,
	`official_url` text,
	`erogamescape_url` text,
	`seiya_url` text,
	`created_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL,
	`updated_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL,
	PRIMARY KEY(`device_id`, `work_id`),
	FOREIGN KEY (`device_id`) REFERENCES `devices`(`device_id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `idx_device_work_snapshots_device_id` ON `device_work_snapshots` (`device_id`);--> statement-breakpoint
CREATE INDEX `idx_device_work_snapshots_device_id_erogamescape_id` ON `device_work_snapshots` (`device_id`,`erogamescape_id`);--> statement-breakpoint
CREATE TABLE `devices` (
	`device_id` text PRIMARY KEY NOT NULL,
	`secret_hash` text NOT NULL,
	`created_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL,
	`updated_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL,
	`last_synced_at` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `devices_secret_hash_unique` ON `devices` (`secret_hash`);--> statement-breakpoint
CREATE TABLE `remote_share_images` (
	`dedupe_key` text PRIMARY KEY NOT NULL,
	`image_key` text NOT NULL,
	`created_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL,
	`updated_at` text DEFAULT CURRENT_TIMESTAMP NOT NULL
);
