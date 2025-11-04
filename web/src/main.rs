use {
    dioxus::{logger::tracing, prelude::*},
    ui::{AppAccess, Header, Navbar},
    views::{Blog, Home, KnowledgeSpace, NotFound},
};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/knowledge-space")]
    KnowledgeSpace {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "web")]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    backend::ServerInstance::serve(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}

#[component]
fn MainLayout() -> Element {
    let navigation = navigator();

    rsx! {
        Header {
            left: rsx! {
                Navbar {
                    Link { to: Route::Home {}, "Home" }
                    Link { to: Route::Blog { id: 1 }, "Blog" }
                }
            },
            right: rsx! {
                AppAccess {
                    handle_log_in: move |_| tracing::debug!("Log in"),
                    handle_free_trial: move |_| {
                        navigation.push(Route::KnowledgeSpace {});
                    },
                }
            },
        }

        Outlet::<Route> {}
    }
}
