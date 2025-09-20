use domain::scan::{DuplicateResolver, ResolvedWork};

pub struct HeuristicDuplicateResolver;

impl HeuristicDuplicateResolver {
    const IGNORE_WORD_WHEN_CONFLICT: [&'static str; 29] = [
        "設定",
        "チェック",
        "インスト",
        "削除",
        "ファイル",
        "ください",
        "下さい",
        "マニュアル",
        "アップデート",
        "システム",
        "check",
        "setting",
        "config",
        "update",
        "inst",
        "tool",
        "support",
        "setup",
        "unins",
        "define",
        "bhvc",
        "bootstrap",
        "file",
        "exhibit",
        "ihs",
        "launcher",
        "syscfg",
        "updchk",
        "acmp",
    ];
    const SHOULD_UPDATE_WORD_WHEN_CONFLICT: [&'static str; 6] =
        ["adv", "64", "cmvs", "bgi", "実行", "起動"];

    fn grouping_key(item: &ResolvedWork) -> i32 {
        item.egs_id
    }

    fn better(a_text: &str, b_text: &str, a_distance: f32, b_distance: f32) -> bool {
        // returns true if b is better than a
        let mut must_update = false;
        let mut not_must_update = false;
        for w in Self::IGNORE_WORD_WHEN_CONFLICT {
            if a_text.contains(w) {
                must_update = true;
                break;
            }
            if b_text.contains(w) {
                not_must_update = true;
                break;
            }
        }
        for w in Self::SHOULD_UPDATE_WORD_WHEN_CONFLICT {
            if a_text.contains(w) {
                not_must_update = true;
                break;
            }
            if b_text.contains(w) {
                must_update = true;
                break;
            }
        }
        if must_update && !not_must_update {
            return true;
        }
        if !not_must_update {
            if a_distance < b_distance {
                return true;
            }
        }
        false
    }
}

impl DuplicateResolver for HeuristicDuplicateResolver {
    fn resolve(&self, items: Vec<ResolvedWork>) -> Vec<ResolvedWork> {
        use std::collections::HashMap;
        let mut groups: HashMap<i32, Vec<ResolvedWork>> = HashMap::new();
        for it in items {
            groups.entry(Self::grouping_key(&it)).or_default().push(it);
        }
        let mut out: Vec<ResolvedWork> = Vec::new();
        for (_key, mut vec) in groups.into_iter() {
            // key(erogamescape id) に対して、得られた resolved_work が1つならそれをそのまま返す
            if vec.len() == 1 {
                out.push(vec.pop().unwrap());
                continue;
            }

            // 複数ある場合は、キーワードとタイトルの距離を比較して最適なものを選択
            let mut best_idx: usize = 0;
            let mut best_path = vec[0].candidate.path.to_string_lossy().to_string();
            let mut best_distance: f32 = vec[0].distance;
            for (idx, item) in vec.iter().enumerate().skip(1) {
                let path = item.candidate.path.to_string_lossy().to_string();
                if Self::better(&best_path, &path, best_distance, item.distance) {
                    best_idx = idx;
                    best_path = path;
                    best_distance = item.distance;
                }
            }
            out.push(vec.swap_remove(best_idx));
        }
        out
    }
}

#[cfg(test)]
mod test;
