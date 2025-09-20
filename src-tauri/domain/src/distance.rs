pub struct Distance {
    a: Vec<char>,
    b: Vec<char>,
    m: usize,
    n: usize, // m <= n, m is a length, n is b length
}

impl Distance {
    pub fn new(a: &str, b: &str) -> Self {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let (m, n) = (a.len(), b.len());

        if m > n {
            return Distance {
                a: b,
                b: a,
                m: n,
                n: m,
            };
        }

        Distance { a, b, m, n }
    }

    pub fn onp(&self) -> usize {
        let offset: isize = (self.m as isize) + 1;
        let delta: isize = (self.n as isize) - (self.m as isize);
        let mut fp = vec![-1; self.m + self.n + 3];

        let mut p: isize = 0;
        loop {
            // -p <= k <= delta - 1
            for k in (-p)..=(delta - 1) {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            // delta + 1 <= k <= delta + p
            for k in ((delta + 1)..=(delta + p)).rev() {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            // delta == k
            fp[(delta + offset) as usize] = self.snake(
                delta,
                (fp[(delta - 1 + offset) as usize] + 1).max(fp[(delta + 1 + offset) as usize]),
            );
            if fp[(delta + offset) as usize] == (self.n as isize) {
                return (delta + 2 * p) as usize;
            }
            p += 1;
        }
    }

    fn snake(&self, k: isize, y: isize) -> isize {
        let mut x = y - k;
        let mut y = y;
        while x < self.m as isize && y < self.n as isize && self.a[x as usize] == self.b[y as usize]
        {
            x += 1;
            y += 1;
        }
        y
    }

    /// 上限距離を与えて早期終了する ONP。
    /// 最小編集距離が `max_distance` を超えることが確定したら None を返す。
    pub fn onp_bounded(&self, max_distance: usize) -> Option<usize> {
        let offset: isize = (self.m as isize) + 1;
        let delta: isize = (self.n as isize) - (self.m as isize);
        let mut fp = vec![-1; self.m + self.n + 3];

        // 事前に長さ差で到達不能を判定
        if delta < 0 {
            return None;
        }
        if (delta as usize) > max_distance {
            return None;
        }

        let mut p: isize = 0;
        loop {
            let d_now = (delta + 2 * p) as isize;
            if d_now as usize > max_distance {
                return None;
            }

            for k in (-p)..=(delta - 1) {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            for k in ((delta + 1)..=(delta + p)).rev() {
                fp[(k + offset) as usize] = self.snake(
                    k,
                    (fp[(k - 1 + offset) as usize] + 1).max(fp[(k + 1 + offset) as usize]),
                );
            }
            fp[(delta + offset) as usize] = self.snake(
                delta,
                (fp[(delta - 1 + offset) as usize] + 1).max(fp[(delta + 1 + offset) as usize]),
            );
            if fp[(delta + offset) as usize] == (self.n as isize) {
                return Some(d_now as usize);
            }
            p += 1;
        }
    }
}

pub fn get_comparable_distance(a: &str, b: &str) -> f32 {
    let distance = Distance::new(&a, &b);
    let distance_value = distance.onp();

    1.0 - (distance_value as f32 / a.len().max(b.len()) as f32)
}

/// 閾値を用いた上限付きスコア。到達不可能なら None。
pub fn get_comparable_distance_bounded(a: &str, b: &str, min_similarity: f32) -> Option<f32> {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return Some(1.0);
    }

    // s = 1 - d/max_len > min_similarity を満たす整数 d の最大値
    let raw = (1.0 - min_similarity) * (max_len as f32);
    let mut d_max = raw.floor() as isize;
    if (raw - (d_max as f32)).abs() < 1e-6 {
        d_max -= 1;
    }
    if d_max < 0 {
        d_max = 0;
    }

    let distance = Distance::new(a, b);
    let d = distance.onp_bounded(d_max as usize)?;
    let score = 1.0 - (d as f32 / max_len as f32);
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onp() {
        let cases = vec![("abc", "abcdef", 3), ("abc", "ab", 1), ("abc", "abc", 0)];

        for (a, b, expected) in cases {
            let distance = Distance::new(a, b);
            let result = distance.onp();
            assert_eq!(result, expected, "Failed on values a:{}, b:{}", a, b);
        }
    }
}
