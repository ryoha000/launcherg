#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ShellLink {
    fn create_bulk(&self, items: Vec<CreateShortcutRequest>) -> anyhow::Result<()>;
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


