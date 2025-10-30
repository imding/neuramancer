use {crate::Snippet, dioxus::prelude::*, schema::Snippet as SnippetStruct};

#[component]
pub fn NoteSnippets(snippets: Vec<SnippetStruct>) -> Element {
    rsx! {
        div {
            for (index, snippet) in snippets.iter().enumerate() {
                Snippet {
                    key: "snippet-{index}",
                    snippet: snippet.clone()
                }
            }
        }
    }
}
