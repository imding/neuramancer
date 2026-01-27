use {
    dioxus::{logger::tracing, prelude::*},
    ui::{AppAccess, Header, Navbar, StoresProvider},
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
        StoresProvider {
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }

            Router::<Route> {}
        }
    }
}

#[component]
fn MainLayout() -> Element {
    let navigation = navigator();
    let route: Route = use_route();
    let show_knowledge_space = matches!(route, Route::KnowledgeSpace {});

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

        div {
            id: "knowledge-space",
            style: if show_knowledge_space { "display: block;" } else { "display: none;" },

            canvas { id: "bevy-render" }
        }

        Outlet::<Route> {}
    }
}
