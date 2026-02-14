mod server;
mod surreal;

use {
    dioxus::prelude::*,
    schema::{Knot, Note},
};

pub use surreal::NewNote;

#[cfg(feature = "server")]
use {
    dioxus::{fullstack::extract::State, logger::tracing},
    schema::{SnippetData, TextSnippet},
};

#[cfg(feature = "server")]
pub use {
    server::{ServerInstance, ServerState},
    surreal::{SnippetDataOrId, SurrealInMemory, SurrealService},
};

#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[post("/api/notes")]
pub async fn save_note(content: String) -> Result<NewNote, ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;
    let result = state
        .surreal
        .create_note(vec![SnippetDataOrId::Data(SnippetData::TextSnippet(
            TextSnippet {
                content,
                embedding: Vec::new(),
            },
        ))])
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: format!("{error}"),
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            details: None,
        })?;

    Ok(result)
}

#[get("/api/notes")]
pub async fn read_notes() -> Result<Vec<Note>, ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;
    let notes = state
        .surreal
        .read_notes()
        .await
        .map_err(|error| ServerFnError::ServerError {
            message: format!("{error}"),
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            details: None,
        })?;

    Ok(notes)
}

#[post("/api/notes/delete")]
pub async fn delete_note(id: String) -> Result<(), ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;

    match state.surreal.delete_note(id.clone()).await {
        Ok(()) => {
            tracing::debug!("Successfully deleted note with id: {}", id);
            Ok(())
        }
        Err(error) => Err(ServerFnError::ServerError {
            message: format!("Failed to delete note {id}: {error}"),
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            details: None,
        }),
    }
}

#[post("/api/knots")]
pub async fn create_knot(
    label: String,
    intent: String,
    note_ids: Vec<String>,
    knot_ids: Vec<String>,
) -> Result<Knot, ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;

    // Validate input parameters
    if knot_ids.is_empty() && note_ids.is_empty() {
        return Err(ServerFnError::ServerError {
            message: "At least one knot ID or note ID must be provided".to_string(),
            code: StatusCode::BAD_REQUEST.as_u16(),
            details: None,
        });
    }

    if label.trim().is_empty() {
        return Err(ServerFnError::ServerError {
            message: "Label cannot be empty".to_string(),
            code: StatusCode::BAD_REQUEST.as_u16(),
            details: None,
        });
    }

    if intent.trim().is_empty() {
        return Err(ServerFnError::ServerError {
            message: "Intent cannot be empty".to_string(),
            code: StatusCode::BAD_REQUEST.as_u16(),
            details: None,
        });
    }

    match state
        .surreal
        .create_knot(label.clone(), intent.clone(), note_ids, knot_ids)
        .await
    {
        Ok(knot) => Ok(knot),
        Err(error) => {
            tracing::debug!("{error}");
            Err(ServerFnError::ServerError {
                message: error.to_string(),
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                details: None,
            })
        }
    }
}

#[get("/api/knots")]
pub async fn read_knots() -> Result<Vec<Knot>, ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;

    match state.surreal.read_knots().await {
        Ok(knots) => Ok(knots),
        Err(error) => {
            tracing::error!("{error}");
            Err(ServerFnError::ServerError {
                message: error.to_string(),
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                details: None,
            })
        }
    }
}

#[post("/api/knots/delete")]
pub async fn delete_knot(id: String, recursive: bool) -> Result<(), ServerFnError> {
    let State(state): State<ServerState> = FullstackContext::extract().await?;

    match state.surreal.delete_knot(id.clone(), recursive).await {
        Ok(()) => {
            tracing::debug!(
                "Successfully deleted knot with id: {} (recursive: {})",
                id,
                recursive
            );
            Ok(())
        }
        Err(error) => Err(ServerFnError::ServerError {
            message: format!("Failed to delete knot {id}: {error}"),
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            details: None,
        }),
    }
}
