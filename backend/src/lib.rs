use {
    cfg_if::cfg_if,
    dioxus::{logger::tracing, prelude::*},
    schema::{Note, SnippetData},
};

cfg_if! {
    if #[cfg(feature = "server")] {
        use axum::http::StatusCode;

        mod qdrant;
        mod server;
        mod sqlx;
        mod surreal;

        pub use {
            qdrant::*,
            server::*,
            sqlx::*,
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
