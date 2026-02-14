use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        div { class: "grid grid-flow-col justify-start gap-4 p-4 [&_a]:text-white [&_a]:no-underline [&_a]:cursor-pointer",
            {children}
        }
    }
}
