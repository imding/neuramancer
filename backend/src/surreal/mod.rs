use {
    cfg_if::cfg_if,
    schema::{Knot, Note, SnippetData},
    serde::{Deserialize, Serialize},
    std::future::Future,
};

cfg_if! {
    if #[cfg(feature = "server")] {
        mod kv_mem;

        use {
            dioxus::{
                fullstack::response::{IntoResponse, Response},
                prelude::StatusCode,
            },
            serde_json::to_string,
        };

        pub use kv_mem::*;
    }
}

pub enum SnippetDataOrId {
    Data(SnippetData),
    Id((String, String)),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewNote {
    pub id: String,
    pub snippet_ids: Vec<String>,
}

#[cfg(feature = "server")]
impl IntoResponse for NewNote {
    fn into_response(self) -> Response {
        let json_body = match to_string(&self) {
            Ok(body) => body,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to serialize".into())
                    .unwrap();
            }
        };

        Response::builder()
            .status(StatusCode::OK)
            .header("content-Type", "application/json")
            .body(json_body.into())
            .unwrap()
    }
}

pub trait SurrealService {
    type Error;

    fn create_note(
        &self,
        snippets: Vec<SnippetDataOrId>,
    ) -> impl Future<Output = Result<NewNote, Self::Error>> + Send;

    fn read_notes(&self) -> impl Future<Output = Result<Vec<Note>, Self::Error>> + Send;

    fn delete_note(&self, id: String) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_knot(
        &self,
        label: String,
        intent: String,
        note_ids: Vec<String>,
        knot_ids: Vec<String>,
    ) -> impl Future<Output = Result<Knot, Self::Error>> + Send;

    fn read_knots(&self) -> impl Future<Output = Result<Vec<Knot>, Self::Error>> + Send;

    fn delete_knot(
        &self,
        id: String,
        recursive: bool,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
