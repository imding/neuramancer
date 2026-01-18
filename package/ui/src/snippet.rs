use {crate::SnippetData, dioxus::prelude::*, schema::Snippet as SnippetStruct};

#[component]
pub fn Snippet(snippet: SnippetStruct) -> Element {
    let data = snippet.data;

    match snippet.id_ {
        Some(id) => rsx! {
            p { "Snippet: {id}" }

            SnippetData { data }
        },
        _ => rsx! {
            SnippetData { data }
        },
    }
}
