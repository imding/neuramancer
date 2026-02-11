use {crate::Route, dioxus::prelude::*};

#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { class: "mt-[50px] pointer-events-auto",

            // Content
            h1 { "This is blog #{id}!" }
            p {
                "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
            }

            // Navigation links
            Link { class: "text-white mt-[50px]", to: Route::Blog { id: id - 1 }, "Previous" }
            span { " <---> " }
            Link { class: "text-white mt-[50px]", to: Route::Blog { id: id + 1 }, "Next" }
        }
    }
}
