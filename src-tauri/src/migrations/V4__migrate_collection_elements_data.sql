-- データ移行: 既存のcollection_elementsから新しいテーブルにデータを移行

-- 1. collection_element_info_by_erogamescapeにデータ移行
-- gamename + collection_element_detailsの内容を統合
INSERT INTO collection_element_info_by_erogamescape (
    collection_element_id, 
    gamename, 
    gamename_ruby, 
    sellday, 
    is_nukige, 
    brandname, 
    brandname_ruby,
    created_at,
    updated_at
)
SELECT 
    ce.id,
    ce.gamename,
    COALESCE(ced.gamename_ruby, ''),
    COALESCE(ced.sellday, ''),
    COALESCE(ced.is_nukige, 0),
    COALESCE(ced.brandname, ''),
    COALESCE(ced.brandname_ruby, ''),
    ce.created_at,
    ce.updated_at
FROM collection_elements ce
LEFT JOIN collection_element_details ced ON ce.id = ced.collection_element_id;

-- 2. collection_element_pathsにデータ移行
INSERT INTO collection_element_paths (
    collection_element_id,
    exe_path,
    lnk_path,
    created_at,
    updated_at
)
SELECT 
    id,
    exe_path,
    lnk_path,
    created_at,
    updated_at
FROM collection_elements
WHERE exe_path IS NOT NULL OR lnk_path IS NOT NULL;

-- 3. collection_element_installsにデータ移行
INSERT INTO collection_element_installs (
    collection_element_id,
    install_at,
    created_at,
    updated_at
)
SELECT 
    id,
    install_at,
    created_at,
    updated_at
FROM collection_elements
WHERE install_at IS NOT NULL;

-- 4. collection_element_playsにデータ移行
INSERT INTO collection_element_plays (
    collection_element_id,
    last_play_at,
    created_at,
    updated_at
)
SELECT 
    id,
    last_play_at,
    created_at,
    updated_at
FROM collection_elements
WHERE last_play_at IS NOT NULL;

-- 5. collection_element_likesにデータ移行
INSERT INTO collection_element_likes (
    collection_element_id,
    like_at,
    created_at,
    updated_at
)
SELECT 
    id,
    like_at,
    created_at,
    updated_at
FROM collection_elements
WHERE like_at IS NOT NULL;

-- 6. collection_element_thumbnailsにデータ移行
INSERT INTO collection_element_thumbnails (
    collection_element_id,
    thumbnail_width,
    thumbnail_height,
    created_at,
    updated_at
)
SELECT 
    id,
    thumbnail_width,
    thumbnail_height,
    created_at,
    updated_at
FROM collection_elements
WHERE thumbnail_width IS NOT NULL OR thumbnail_height IS NOT NULL;