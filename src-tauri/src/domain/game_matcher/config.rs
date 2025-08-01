/// ゲーム候補検索で使用する設定値

/// ゲーム以外を表す単語（完全一致）
pub const NOT_GAME_EQUALLY_WORD: [&str; 1] = ["bgi"];

/// ゲーム以外を表す語句（部分一致）
pub const NOT_GAME_TERMS: [&str; 16] = [
    "マニュアル",
    "詳細設定",
    "はじめに",
    "サポート",
    "セーブデータ",
    "インストール",
    "アンインストール",
    "体験版",
    "install",
    "uninstall",
    "autorun",
    "削除",
    "license",
    "ライセンス",
    "公式サイト",
    "ホームページ",
];

/// ファイル名から除去する語句
pub const REMOVE_WORDS: [&str; 9] = [
    "を起動",
    "の起動",
    "_単独動作版",
    "「",
    "」",
    " ",
    "　",
    "ダウンロード版",
    "DL版",
];

/// 無視するゲームID
pub const IGNORE_GAME_ID: [i32; 4] = [2644, 63, 2797, 10419];

/// ファイル名と完全一致する場合のゲームIDマッピング
/// (filename, game_id)の配列
pub const EQUALLY_FILENAME_GAME_ID_PAIR: [(&str, i32); 1] = [("pieces", 27123)];

/// ゲーム以外のファイルかどうかを判定
pub fn is_not_game(filename: &str) -> bool {
    for not_game_str in NOT_GAME_TERMS {
        if filename.contains(not_game_str) {
            return true;
        }
    }
    for not_game_str in NOT_GAME_EQUALLY_WORD {
        if filename.eq(not_game_str) {
            return true;
        }
    }
    false
}

/// ファイル名から不要な語句を除去
pub fn remove_unnecessary_words(filename: &str) -> String {
    let mut result = filename.to_string();
    for word in REMOVE_WORDS.iter() {
        result = result.replace(word, "");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_not_game() {
        assert!(is_not_game("マニュアル.exe"));
        assert!(is_not_game("install.exe"));
        assert!(is_not_game("bgi"));
        assert!(!is_not_game("game.exe"));
    }

    #[test]
    fn test_remove_unnecessary_words() {
        assert_eq!(remove_unnecessary_words("ゲームを起動"), "ゲーム");
        assert_eq!(remove_unnecessary_words("test DL版"), "test");
        assert_eq!(remove_unnecessary_words("「テスト」"), "テスト");
    }
}