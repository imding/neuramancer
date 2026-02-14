use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div { class: "w-[360px] mx-auto mt-[50px] bg-[#1e222d] p-5 rounded-[10px]",
            h4 { class: "mb-[15px]", "ServerFn Echo" }
            input {
                class: "border-b border-b-white bg-transparent text-white outline-none block w-full pb-[5px] focus:border-b-[#6d85c6] transition-colors duration-200",
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = backend::echo(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p { class: "mt-5 ml-auto",
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}
