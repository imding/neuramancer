use {
    dioxus::prelude::*,
    ui::{Demo, Hero},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "grid justify-center gap-8",

            Hero {}
            Demo {}
        }
    }
}
