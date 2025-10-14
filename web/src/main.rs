use {
    dioxus::prelude::*,
    ui::{AppAccess, Header, Navbar},
    views::{Blog, Home, NotFound, Notes},
};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Home {},
    #[route("/notes")]
    Notes {},
    #[route("/blog/:id")]
    Blog { id: i32 },
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
    rsx! {
        Header {
            left: rsx! {
                Navbar {
                    Link {
                        to: Route::Home {},
                        "Home"
                    }
                    Link {
                        to: Route::Blog { id: 1 },
                        "Blog"
                    }
                }
            },
            right: rsx! {
                AppAccess {}
            }
        }

        Outlet::<Route> {}
    }
}
