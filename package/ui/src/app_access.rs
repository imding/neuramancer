use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct AppAccessProps {
    handle_log_in: Option<EventHandler<MouseEvent>>,
    handle_free_trial: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn AppAccess(props: AppAccessProps) -> Element {
    rsx! {
        div { class: "justify-self-end",

            button {
                class: "bg-transparent text-white text-base py-4 px-6 border-none cursor-pointer",
                onclick: props.handle_log_in.unwrap_or_default(),

                "Log-in"
            }

            button {
                class: "text-base py-4 px-6 border-none cursor-pointer",
                onclick: props.handle_free_trial.unwrap_or_default(),

                "Try for free"
            }
        }
    }
}
