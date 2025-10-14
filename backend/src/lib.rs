//! This crate contains all shared fullstack server functions.
use {cfg_if::cfg_if, dioxus::prelude::*};

cfg_if! {
    if #[cfg(feature = "server")] {
        mod qdrant;
        mod server;

        pub use {qdrant::*, server::*};
    }
}

/// Echo the user input on the server.
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(SaveTextNote)]
pub async fn save_text_note(content: String) -> Result<bool, ServerFnError> {
    Ok(true)
}
