use std::path::Path;
use super::normalizer::normalize;
use super::config::{is_not_game, remove_unnecessary_words};

/// ファイルパスから抽出されるマッチング用の情報
#[derive(Debug, Clone, PartialEq)]
pub struct FileMatchingInfo {
    /// 正規化済みファイル名（拡張子なし、不要語句除去済み）
    pub filename: String,
    /// 正規化済み親ディレクトリ名
    pub parent_dir: String,
    /// ファイル名のマッチングをスキップするかどうか
    /// "game"や"start"など汎用的すぎる名前の場合はtrue
    pub skip_filename: bool,
}

/// ファイル名から拡張子を除去
pub fn get_file_name_without_extension(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            let file_name_without_extension = Path::new(file_name_str)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned());
            return file_name_without_extension;
        }
    }
    None
}

/// ファイルパスからマッチング用の情報を抽出
pub fn extract_file_info(filepath: &str) -> anyhow::Result<FileMatchingInfo> {
    // 親ディレクトリ名を取得
    let parent_dir = Path::new(&filepath)
        .parent()
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        })
        .ok_or(anyhow::anyhow!("cannot get parent directory"))?;

    // ファイル名を取得（拡張子なし）
    let filename = get_file_name_without_extension(filepath)
        .ok_or(anyhow::anyhow!("cannot get filename"))?;
    
    let normalized_filename = normalize(&filename);
    
    // ゲーム以外のファイルは除外
    if is_not_game(&normalized_filename) {
        return Err(anyhow::anyhow!("file is not a game"));
    }
    
    // 不要な語句を除去
    let cleaned_filename = remove_unnecessary_words(&normalized_filename);
    
    // 汎用的すぎるファイル名の場合はファイル名マッチングをスキップ
    let skip_filename = cleaned_filename == "game" || cleaned_filename == "start";

    Ok(FileMatchingInfo {
        filename: cleaned_filename,
        parent_dir,
        skip_filename,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_name_without_extension() {
        assert_eq!(
            get_file_name_without_extension("C:\\test\\game.exe"),
            Some("game".to_string())
        );
        assert_eq!(
            get_file_name_without_extension("/test/file.tar.gz"),
            Some("file.tar".to_string())
        );
    }

    #[test]
    fn test_extract_file_info() {
        let result = extract_file_info("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe").unwrap();
        assert_eq!(result.filename, "pieces");
        assert_eq!(result.parent_dir, "pieces");
        assert!(!result.skip_filename);

        let result = extract_file_info("C:\\Program Files\\Game\\start.exe").unwrap();
        assert_eq!(result.filename, "start");
        assert_eq!(result.parent_dir, "game");
        assert!(result.skip_filename);
    }

    #[test]
    fn test_extract_file_info_not_game() {
        let result = extract_file_info("C:\\test\\install.exe");
        assert!(result.is_err());
    }
}