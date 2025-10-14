//! This crate contains all shared fullstack server functions.
use {
    cfg_if::cfg_if,
    dioxus::{logger::tracing, prelude::*},
};

cfg_if! {
    if #[cfg(feature = "server")] {
        mod qdrant;
        mod server;
        mod sqlx;

        pub use {qdrant::*, server::*, sqlx::*};
    }
}

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(SaveTextNote)]
pub async fn save_text_note(content: String) -> Result<bool, ServerFnError> {
    let FromContext(state): FromContext<ServerState> = match extract().await {
        Ok(state) => state,
        Err(error) => {
            tracing::error!("{error}");
            return Ok(false);
        }
    };
    let new_note = state.sqlx.create_text_note(&content).await?;

    tracing::debug!("{new_note:?}");

    Ok(true)
}
