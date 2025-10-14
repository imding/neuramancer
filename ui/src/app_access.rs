use dioxus::prelude::*;

const APP_ACCESS_CSS: Asset = asset!("/assets/styling/app_access.css");

#[component]
pub fn AppAccess() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: APP_ACCESS_CSS }

        div {
            id: "app-access",

            button {
                id: "log-in",

                "Log-in"
            }

            button {
                id: "free-trial",

                "Try for free"
            }
        }
    }
}
