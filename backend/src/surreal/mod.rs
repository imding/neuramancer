mod kv_mem;

pub use kv_mem::*;

use {
    schema::{Note, SnippetData},
    std::future::Future,
};

pub trait SurrealService {
    type Error;

    fn create_note(
        &self,
        snippets: Vec<SnippetData>,
    ) -> impl Future<Output = Result<Note, Self::Error>> + Send;

    fn read_notes(&self) -> impl Future<Output = Result<Vec<Note>, Self::Error>> + Send;
}
