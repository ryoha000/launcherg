/// 文字列を正規化する関数
/// 全角英数字を半角に変換し、小文字化する
pub fn normalize(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch >= 'Ａ' && ch <= 'Ｚ' || ch >= 'ａ' && ch <= 'ｚ' {
            result.push((ch as u32 - 'Ａ' as u32 + 'A' as u32) as u8 as char);
        } else if ch >= '０' && ch <= '９' {
            result.push((ch as u32 - '０' as u32 + '0' as u32) as u8 as char);
        } else {
            result.push(ch);
        }
    }
    result.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("ＡＢＣ"), "abc");
        assert_eq!(normalize("１２３"), "123");
        assert_eq!(normalize("Test１２３"), "test123");
        assert_eq!(normalize("テスト"), "テスト");
    }
}