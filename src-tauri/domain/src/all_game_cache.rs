#[derive(derive_new::new, Debug, Clone)]
pub struct AllGameCacheOne {
    pub id: i32,
    pub gamename: String,
}

#[derive(derive_new::new, Debug, Clone)]
pub struct NewAllGameCacheOne {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone)]
pub struct AllGameCacheOneWithThumbnailUrl {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

pub type AllGameCache = Vec<AllGameCacheOne>;
