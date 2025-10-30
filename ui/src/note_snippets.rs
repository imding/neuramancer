use {crate::Snippet, dioxus::prelude::*, schema::Snippet as SnippetStruct};

#[component]
pub fn NoteSnippets(snippets: Vec<SnippetStruct>) -> Element {
    rsx! {
        div {
            for snippet in snippets {
                Snippet { snippet }
            }
        }
    }
}
