pub mod transaction;
pub mod works;

pub trait RepositoriesExt {
    type WorkRepo: works::WorkRepository;
    type TransactionRepo: transaction::TransactionRepository;

    fn work(&mut self) -> &mut Self::WorkRepo;
    fn transaction(&mut self) -> &mut Self::TransactionRepo;
}
