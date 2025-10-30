use dioxus::prelude::*;

const APP_ACCESS_CSS: Asset = asset!("/assets/styling/app_access.css");

#[derive(Clone, PartialEq, Props)]
pub struct AppAccessProps {
    handle_log_in: Option<EventHandler<MouseEvent>>,
    handle_free_trial: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn AppAccess(props: AppAccessProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: APP_ACCESS_CSS }

        div {
            id: "app-access",

            button {
                id: "log-in",
                onclick: props.handle_log_in.unwrap_or_default(),

                "Log-in"
            }

            button {
                id: "free-trial",
                onclick: props.handle_free_trial.unwrap_or_default(),

                "Try for free"
            }
        }
    }
}
