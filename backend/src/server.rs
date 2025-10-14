use {
    crate::{Qdrant, SqliteInMemory, SurrealInMemory},
    dioxus::prelude::*,
    once_cell::sync::Lazy,
    tokio::runtime::Runtime,
};

static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Tokio runtime should lazily initialise."));

static SERVER_STATE: Lazy<ServerState> =
    Lazy::new(|| RUNTIME.block_on(async { ServerState::new().await }));

pub struct ServerInstance;

#[derive(Clone)]
pub struct ServerState {
    pub sqlx: SqliteInMemory,
    pub qdrant: Qdrant,
    pub surreal: SurrealInMemory,
}

impl ServerState {
    pub async fn new() -> Self {
        ServerState {
            sqlx: match SqliteInMemory::new().await {
                Ok(sqlx) => sqlx,
                Err(error) => panic!("{error:?}"),
            },
            qdrant: match Qdrant::new(None, Some("http://localhost:6334")).await {
                Ok(qdrant) => qdrant,
                Err(error) => panic!("{error:?}"),
            },
            surreal: match SurrealInMemory::init().await {
                Ok(surreal) => surreal,
                Err(error) => panic!("{error:?}"),
            },
        }
    }
}

impl ServerInstance {
    pub fn serve(component: fn() -> Element) {
        LaunchBuilder::new()
            .with_context(SERVER_STATE.clone())
            .launch(component);
    }
}
