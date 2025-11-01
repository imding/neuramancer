mod kv_mem;

pub use kv_mem::*;

use {
    schema::{Knot, Note, SnippetData},
    std::future::Future,
};

pub trait SurrealService {
    type Error;

    fn create_note(
        &self,
        snippets: Vec<SnippetData>,
    ) -> impl Future<Output = Result<Note, Self::Error>> + Send;

    fn read_notes(&self) -> impl Future<Output = Result<Vec<Note>, Self::Error>> + Send;

    fn delete_note(
        &self,
        id: String,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_knot(
        &self,
        knot_ids: Vec<String>,
        note_ids: Vec<String>,
        intent: String,
    ) -> impl Future<Output = Result<Knot, Self::Error>> + Send;

    fn read_knots(&self) -> impl Future<Output = Result<Vec<Knot>, Self::Error>> + Send;

    fn delete_knot(
        &self,
        id: String,
        recursive: bool,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
