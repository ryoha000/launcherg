#[derive(Debug, Clone)]
pub struct AllGameCacheOne {
    pub id: i32,
    pub gamename: String,
}

#[derive(Debug, Clone)]
pub struct NewAllGameCacheOne {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

pub type AllGameCache = Vec<AllGameCacheOne>;
