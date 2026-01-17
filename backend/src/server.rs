use {
    crate::SurrealInMemory,
    dioxus::{
        fullstack::{extract::FromRef, FullstackContext},
        prelude::*,
    },
};

pub struct ServerInstance;

#[derive(Clone)]
pub struct ServerState {
    // pub sqlx: SqliteInMemory,
    // pub qdrant: Qdrant,
    pub surreal: SurrealInMemory,
}

impl ServerState {
    pub async fn new() -> Self {
        ServerState {
            // sqlx: match SqliteInMemory::new().await {
            //     Ok(sqlx) => sqlx,
            //     Err(error) => panic!("{error:?}"),
            // },
            // qdrant: match Qdrant::new(None, Some("http://localhost:6334")).await {
            //     Ok(qdrant) => qdrant,
            //     Err(error) => panic!("{error:?}"),
            // },
            surreal: match SurrealInMemory::init().await {
                Ok(surreal) => surreal,
                Err(error) => panic!("{error:?}"),
            },
        }
    }
}

impl FromRef<FullstackContext> for ServerState {
    fn from_ref(ctx: &FullstackContext) -> Self {
        ctx.extension::<ServerState>().unwrap()
    }
}

impl ServerInstance {
    pub fn serve(component: fn() -> Element) {
        dioxus::serve(|| async move {
            use dioxus::server::axum::Extension;

            // Initialize state inside the existing async runtime to avoid nested Tokio runtimes.
            let server_state = ServerState::new().await;

            let router = dioxus::server::router(component).layer(Extension(server_state));

            Ok(router)
        });
    }
}
