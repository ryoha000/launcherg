use std::collections::{HashMap, HashSet};

use crate::all_game_cache::AllGameCache;

pub type BigramKey = u64;

#[inline]
pub fn encode_bigram(a: char, b: char) -> BigramKey { ((a as u64) << 32) | (b as u64) }

#[derive(Debug, Clone)]
pub struct NGramIndex {
    pub q: usize,
    pub postings: HashMap<BigramKey, Vec<i32>>, // 2-gram -> ids
    pub id_to_len: HashMap<i32, usize>,         // id -> char length
    pub id_to_pos: HashMap<i32, usize>,         // id -> position in snapshot
}

impl NGramIndex {
    pub fn build(snapshot: &AllGameCache, q: usize) -> Self {
        let mut postings: HashMap<BigramKey, Vec<i32>> = HashMap::new();
        let mut id_to_len: HashMap<i32, usize> = HashMap::new();
        let mut id_to_pos: HashMap<i32, usize> = HashMap::new();

        for (pos, g) in snapshot.iter().enumerate() {
            id_to_len.insert(g.id, g.gamename.chars().count());
            id_to_pos.insert(g.id, pos);
            let chars: Vec<char> = g.gamename.chars().collect();
            if chars.len() < q { continue; }
            let mut seen: HashSet<BigramKey> = HashSet::new();
            for i in 0..(chars.len() - (q - 1)) {
                let k = encode_bigram(chars[i], chars[i + 1]);
                if seen.insert(k) {
                    postings.entry(k).or_default().push(g.id);
                }
            }
        }

        for (_k, v) in postings.iter_mut() {
            v.sort_unstable();
            v.dedup();
        }

        Self { q, postings, id_to_len, id_to_pos }
    }

    pub fn filter_candidates(&self, query: &str, min_similarity: f32, ignore_game_ids: &[i32]) -> Vec<i32> {
        let q = self.q;
        let query_chars: Vec<char> = query.chars().collect();
        let query_len = query_chars.len();

        // クエリのユニーク q-gram
        let mut grams: HashSet<BigramKey> = HashSet::new();
        if query_len >= q {
            for i in 0..(query_len - (q - 1)) {
                grams.insert(encode_bigram(query_chars[i], query_chars[i + 1]));
            }
        }

        // d_max を算出
        let calc_dmax = |ly: usize| -> usize {
            let max_len = query_len.max(ly);
            if max_len == 0 { return 0; }
            let raw = (1.0 - min_similarity) * (max_len as f32);
            let mut d = raw.floor() as isize;
            if (raw - (d as f32)).abs() < 1e-6 { d -= 1; }
            if d < 0 { 0 } else { d as usize }
        };

        // 重複 q-gram を排除した overlap 集計
        let mut overlap_counts: HashMap<i32, u16> = HashMap::new();
        for g in grams.iter() {
            if let Some(ids) = self.postings.get(g) {
                for &id in ids {
                    *overlap_counts.entry(id).or_insert(0) += 1;
                }
            }
        }

        let mut candidates: Vec<i32> = Vec::new();
        for (&id, &overlap) in overlap_counts.iter() {
            if ignore_game_ids.contains(&id) { continue; }
            let ly = *self.id_to_len.get(&id).unwrap_or(&0);
            let dmax = calc_dmax(ly);
            let ly_q = ly.saturating_sub(q - 1);
            let lq_unique = grams.len();
            let mut t = (lq_unique.min(ly_q) as isize) - (q as isize * dmax as isize);
            if t <= 0 { t = 0; }
            if (overlap as isize) >= t { candidates.push(id); }
        }
        candidates
    }
}


