-- クリーンアップ: 移行済みテーブルの削除

-- 1. collection_elements_oldテーブルを削除
-- (V4でデータ移行済み)
DROP TABLE IF EXISTS collection_elements_old;

-- 2. collection_element_detailsテーブルを削除
-- (データはcollection_element_info_by_erogamescapeに移行済み)
DROP TABLE IF EXISTS collection_element_details;