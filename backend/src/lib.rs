use {
    cfg_if::cfg_if,
    dioxus::prelude::{server_fn::codec::Json, *},
    schema::{Knot, Note},
};

#[cfg(feature = "server")]
use {axum::http::StatusCode, dioxus::logger::tracing, schema::SnippetData};

cfg_if! {
    if #[cfg(feature = "server")] {
        mod server;
        mod surreal;

        pub use {
            server::*,
            surreal::*,
        };
    }
}

#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(SaveTextNote)]
pub async fn save_note(content: String) -> Result<Note, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };
    let result = state
        .surreal
        .create_note(vec![SnippetData::TextSnippet(schema::TextSnippet {
            content,
        })])
        .await?;

    tracing::debug!("{result:?}");

    Ok(result)
}

#[server(ReadTextNotes)]
pub async fn read_notes() -> Result<Vec<Note>, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };
    let notes = match state.surreal.read_notes().await {
        Ok(notes) => notes,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    Ok(notes)
}

#[server(DeleteNote)]
pub async fn delete_note(id: String) -> Result<(), ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    match state.surreal.delete_note(id.clone()).await {
        Ok(()) => {
            tracing::debug!("Successfully deleted note with id: {}", id);
            Ok(())
        }
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            Err(ServerFnError::ServerError(format!(
                "Failed to delete note {id}: {error}"
            )))
        }
    }
}

#[server(CreateKnot, input = Json)]
pub async fn create_knot(
    label: String,
    intent: String,
    note_ids: Vec<String>,
    knot_ids: Vec<String>,
) -> Result<Knot, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    // Validate input parameters
    if knot_ids.is_empty() && note_ids.is_empty() {
        server_context().response_parts_mut().status = StatusCode::BAD_REQUEST;
        return Err(ServerFnError::ServerError(
            "At least one knot ID or note ID must be provided".to_string(),
        ));
    }

    if label.trim().is_empty() {
        server_context().response_parts_mut().status = StatusCode::BAD_REQUEST;
        return Err(ServerFnError::ServerError(
            "Label cannot be empty".to_string(),
        ));
    }

    if intent.trim().is_empty() {
        server_context().response_parts_mut().status = StatusCode::BAD_REQUEST;
        return Err(ServerFnError::ServerError(
            "Intent cannot be empty".to_string(),
        ));
    }

    match state
        .surreal
        .create_knot(label.clone(), intent.clone(), note_ids, knot_ids)
        .await
    {
        Ok(knot) => Ok(knot),
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            tracing::debug!("{error}");
            Err(ServerFnError::ServerError(error.to_string()))
        }
    }
}

#[server(ReadKnots)]
pub async fn read_knots() -> Result<Vec<Knot>, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            tracing::error!("{error}");
            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    match state.surreal.read_knots().await {
        Ok(knots) => Ok(knots),
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            tracing::error!("{error}");
            Err(ServerFnError::ServerError(error.to_string()))
        }
    }
}

#[server(DeleteKnot)]
pub async fn delete_knot(id: String, recursive: bool) -> Result<(), ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    match state.surreal.delete_knot(id.clone(), recursive).await {
        Ok(()) => {
            tracing::debug!(
                "Successfully deleted knot with id: {} (recursive: {})",
                id,
                recursive
            );
            Ok(())
        }
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            Err(ServerFnError::ServerError(format!(
                "Failed to delete knot {id}: {error}"
            )))
        }
    }
}
