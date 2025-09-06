#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ShellLink {
    fn create_bulk(&self, items: Vec<CreateShortcutRequest>) -> anyhow::Result<()>;
    /// 指定した .lnk / .url の各ファイルパスに対してメタデータを取得する
    ///
    /// 返却値は `HashMap<入力ファイルパス, LnkMetadata>` であり、
    /// `LnkMetadata` の各フィールドは以下を示す:
    /// - `path`:
    ///   - .lnk の場合: ショートカットのリンク先（例: 実体の .exe 絶対パス）
    ///   - .url の場合: .url ファイル自体のパス（URL の実体ではなくファイルパス）
    /// - `icon`:
    ///   - .lnk の場合: IShellLinkW::GetIconLocation により取得されたアイコンファイルパス
    ///   - .url の場合: INI の `IconFile` に記載されたアイコンファイルパス（未設定時は空文字）
    fn get_lnk_metadatas(&self, lnk_file_paths: Vec<String>) -> anyhow::Result<std::collections::HashMap<String, crate::file::LnkMetadata>>;
    fn execute_lnk<'a>(&self, lnk_path: &'a str, is_run_as_admin: bool) -> anyhow::Result<Option<u32>>;
}


#[derive(Clone, Debug)]
pub struct CreateShortcutRequest {
    pub target_path: String,
    pub dest_lnk_path: String,
    pub working_dir: Option<String>,
    pub arguments: Option<String>,
    pub icon_path: Option<String>,
}


