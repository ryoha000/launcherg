CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

CREATE TABLE IF NOT EXISTS collection_elements (
    id INTEGER PRIMARY KEY,
    gamename TEXT NOT NULL,
    exe_path TEXT,
    lnk_path TEXT,
    install_at DATETIME,
    last_play_at DATETIME,
    like_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS collection_element_maps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    collection_element_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    foreign key(collection_id) references collections(id) ON DELETE CASCADE,
    foreign key(collection_element_id) references collection_elements(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS explored_caches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    UNIQUE(path)
);

CREATE TABLE IF NOT EXISTS all_game_caches (
    id INTEGER PRIMARY KEY,
    gamename TEXT NOT NULL,
    thumbnail_url TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS collection_element_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_element_id INTEGER NOT NULL,
    gamename_ruby TEXT NOT NULL,
    sellday TEXT NOT NULL,
    is_nukige INTEGER NOT NULL,
    brandname TEXT NOT NULL,
    brandname_ruby TEXT NOT NULL,
    foreign key(collection_element_id) references collection_elements(id) ON DELETE CASCADE
);
