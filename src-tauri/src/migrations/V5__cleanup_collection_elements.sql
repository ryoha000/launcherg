-- クリーンアップ: 移行済みカラムの削除と不要テーブルの削除

-- 1. collection_elementsから移行済みカラムを削除
-- SQLiteではALTER TABLE DROP COLUMNがサポートされていないため、
-- テーブルを再作成する方法を使用

-- 一時テーブル作成
CREATE TABLE collection_elements_new (
    id INTEGER PRIMARY KEY,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- データをコピー
INSERT INTO collection_elements_new (id, created_at, updated_at)
SELECT id, created_at, updated_at FROM collection_elements;

-- 外部キー制約を一時的に無効化
PRAGMA foreign_keys = OFF;

-- 古いテーブルを削除
DROP TABLE collection_elements;

-- 新しいテーブルを正しい名前にリネーム
ALTER TABLE collection_elements_new RENAME TO collection_elements;

-- 外部キー制約を再有効化
PRAGMA foreign_keys = ON;

-- 2. collection_element_detailsテーブルを削除
-- (データはcollection_element_info_by_erogamescapeに移行済み)
DROP TABLE collection_element_details;