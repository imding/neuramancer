mod views;

use {
    dioxus::{logger::tracing, prelude::*},
    knowledge_space_web::start_bevy,
    ui::{AppAccess, Header, Navbar, StoresProvider},
    views::{Blog, Home, KnowledgeSpace, NotFound},
};

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
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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
            document::Link { rel: "stylesheet", href: TAILWIND_CSS }

            Router::<Route> {}
        }
    }
}

#[component]
fn MainLayout() -> Element {
    let navigation = navigator();
    let mut bevy_started = use_signal(|| false);
    let bevy_ready = use_signal(|| false);

    use_effect(move || {
        if !bevy_started() {
            bevy_started.set(true);
            start_bevy("#bevy-render");
        }
    });

    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, closure::Closure};

            let Some(window) = web_sys::window()
            else {
                return;
            };

            let mut bevy_ready = bevy_ready;
            let handler = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                bevy_ready.set(true);
            }) as Box<dyn FnMut(_)>);

            let _ = window
                .add_event_listener_with_callback("bevy:ready", handler.as_ref().unchecked_ref());

            handler.forget();
        }
    });

    rsx! {
        div { id: "bevy-layer", class: if bevy_ready() { "ready" } else { "" },
            canvas { id: "bevy-render" }
        }

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
