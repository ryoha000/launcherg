pub mod works;

pub trait RepositoriesExt {
    type WorkRepo: works::WorkRepository;

    fn work(&mut self) -> &mut Self::WorkRepo;
}
