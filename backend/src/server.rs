use {
    crate::{Qdrant, SqliteInMemory},
    dioxus::prelude::*,
    tokio::runtime::Runtime,
};

pub struct ServerInstance;

#[derive(Clone)]
pub struct ServerState {
    pub sqlx: SqliteInMemory,
    pub qdrant: Qdrant,
}

impl ServerState {
    pub async fn new() -> Self {
        ServerState {
            sqlx: SqliteInMemory::new().await.unwrap(),
            qdrant: Qdrant::new(None, Some("http://localhost:6334"))
                .await
                .unwrap(),
        }
    }
}

impl ServerInstance {
    pub fn serve(component: fn() -> Element) {
        let server_state = Runtime::new().unwrap().block_on(async move {
            ServerState {
                sqlx: SqliteInMemory::new().await.unwrap(),
                qdrant: Qdrant::new(None, Some("http://localhost:6334"))
                    .await
                    .unwrap(),
            }
        });

        LaunchBuilder::new()
            .with_context(server_state)
            .launch(component);
    }
}
