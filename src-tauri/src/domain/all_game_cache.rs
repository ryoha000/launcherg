#[derive(Debug, Clone)]
pub struct AllGameCacheOne {
    pub id: i32,
    pub gamename: String,
}

pub type AllGameCache = Vec<AllGameCacheOne>;
