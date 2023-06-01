#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
