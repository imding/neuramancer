use {
    crate::Qdrant,
    axum::{Router, serve},
    dioxus::{
        cli_config::{server_ip, server_port},
        prelude::*,
    },
    sqlx::{SqlitePool, sqlite::SqlitePoolOptions},
    std::net::{IpAddr, Ipv4Addr, SocketAddr},
    tokio::{net::TcpListener, runtime::Runtime},
};

pub struct ServerInstance;

#[derive(Clone)]
struct ServerState {
    sqlx: SqlitePool,
    qdrant: Qdrant,
}

impl ServerInstance {
    pub fn launch(component: fn() -> Element) {
        Runtime::new().unwrap().block_on(async move {
            let ip = server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            let port = server_port().unwrap_or(8080);
            let address = SocketAddr::new(ip, port);
            let listener = TcpListener::bind(address).await.unwrap();
            let server_state = ServerState {
                sqlx: SqlitePoolOptions::new()
                    .max_connections(20)
                    .connect_with("sqlite::memory:".parse().unwrap())
                    .await
                    .unwrap(),
                qdrant: Qdrant::new(None, Some("http://localhost:6334"))
                    .await
                    .unwrap(),
            };
            let router = Router::new()
                .serve_dioxus_application(ServeConfigBuilder::default(), component)
                .with_state(server_state)
                .into_make_service();

            serve(listener, router).await.unwrap();
        })
    }
}
