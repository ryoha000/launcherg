use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use domain::all_game_cache::{AllGameCache, AllGameCacheOne};
use domain::game_matcher::matcher::{Matcher, MatcherConfig};
use domain::game_matcher::GameMatcher;
use rand::prelude::*;
use std::time::Duration;

fn japanese_char_pool() -> Vec<char> {
    let mut pool = Vec::new();
    // ひらがな
    pool.extend(('ぁ'..='ん').into_iter());
    // カタカナ
    pool.extend(('ァ'..='ン').into_iter());
    // 長音・中黒
    pool.push('ー');
    pool.push('・');
    pool
}

fn random_japanese_string(rng: &mut impl Rng, len: usize, pool: &[char]) -> String {
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..pool.len());
            pool[idx]
        })
        .collect()
}

fn build_cache(num: usize, name_len: usize, pool: &[char]) -> AllGameCache {
    let mut rng = rand::thread_rng();
    let mut v = Vec::with_capacity(num);
    for i in 0..num {
        let gamename = random_japanese_string(&mut rng, name_len, pool);
        v.push(AllGameCacheOne::new(i as i32, gamename));
    }
    v
}

fn mutate_similar(name: &str, rng: &mut impl Rng, pool: &[char]) -> String {
    if name.is_empty() {
        return name.to_string();
    }
    let mut chars: Vec<char> = name.chars().collect();
    let pos = rng.gen_range(0..chars.len());
    let mut new_c = chars[pos];
    // 別文字になるまで選び直す
    for _ in 0..4 {
        let idx = rng.gen_range(0..pool.len());
        if pool[idx] != chars[pos] {
            new_c = pool[idx];
            break;
        }
    }
    chars[pos] = new_c;
    chars.into_iter().collect()
}

fn build_queries(
    cache: &AllGameCache,
    total: usize,
    name_len: usize,
    pool: &[char],
) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut queries: Vec<String> = (0..total)
        .map(|_| random_japanese_string(&mut rng, name_len, pool))
        .collect();

    // 0.1% をポジティブ（厳密一致 or 近似）にする
    let positives = (total / 1000).max(1);

    // ユニークなインデックスを選ぶ
    let indices = rand::seq::index::sample(&mut rng, total, positives).into_vec();
    for (i, idx) in indices.into_iter().enumerate() {
        // キャッシュからランダムに1つ選ぶ
        let game_idx = rng.gen_range(0..cache.len());
        let target = &cache[game_idx].gamename;
        // 半々で厳密一致 or 近似
        let q = if i % 2 == 0 {
            target.clone()
        } else {
            mutate_similar(target, &mut rng, pool)
        };
        queries[idx] = q;
    }
    queries
}

fn bench_find_candidates(c: &mut Criterion) {
    let mut group = c.benchmark_group("matcher_find_candidates_mixed");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(10));

    // 固定: キャッシュ 40,000 件、日本語名 20 文字
    let name_len = 20;
    let pool = japanese_char_pool();
    let cache = build_cache(40_000, name_len, &pool);
    let mut config = MatcherConfig::default();
    config.similarity_threshold = 0.8; // 既定値に近い

    // 指定のクエリ数
    let sizes = [100, 500];
    // let sizes = [1_000usize, 5_000, 10_000/* , 50_000, 100_000, 200_000, 500_000 */];
    for &qsize in &sizes {
        let matcher = Matcher::new(cache.clone(), config.clone());
        let queries = build_queries(&cache, qsize, name_len, &pool);

        group.throughput(Throughput::Elements(qsize as u64));
        group.bench_with_input(BenchmarkId::new("mixed", qsize), &qsize, |b, &_s| {
            b.iter(|| {
                matcher.clear_cache();
                let _ = matcher.find_candidates(&queries);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_find_candidates);
criterion_main!(benches);
