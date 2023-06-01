use derive_new::new;

use crate::domain::collection::NewCollection;

#[derive(new)]
pub struct CreateCollection {
    pub name: String,
}

impl TryFrom<CreateCollection> for NewCollection {
    type Error = anyhow::Error;

    fn try_from(c: CreateCollection) -> anyhow::Result<Self> {
        Ok(NewCollection::new(c.name))
    }
}
