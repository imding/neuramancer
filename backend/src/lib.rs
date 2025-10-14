use {cfg_if::cfg_if, dioxus::prelude::*, schema::TextNote};

cfg_if! {
    if #[cfg(feature = "server")] {
        use axum::http::StatusCode;

        mod qdrant;
        mod server;
        mod sqlx;

        pub use {qdrant::*, server::*, sqlx::*};
    }
}

#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(SaveTextNote)]
pub async fn save_text_note(content: String) -> Result<TextNote, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    match state.sqlx.create_text_note(&content).await {
        Ok(text_note) => Ok(text_note),
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    }
}

#[server(ReadTextNotes)]
pub async fn read_text_notes() -> Result<Vec<TextNote>, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::SERVICE_UNAVAILABLE;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };
    let notes = match state.sqlx.read_text_notes().await {
        Ok(new_note) => new_note,
        Err(error) => {
            server_context().response_parts_mut().status = StatusCode::INTERNAL_SERVER_ERROR;

            return Err(ServerFnError::ServerError(format!("{error}")));
        }
    };

    Ok(notes)
}
