use {
    dioxus::prelude::*,
    schema::{Note as NoteStruct, Snippet as SnippetStruct, SnippetData as SnippetDataEnum},
};

#[component]
pub fn NoteList() -> Element {
    let notes = use_server_future(move || backend::read_notes())?;

    rsx! {
        div {
            id: "note-list",

            for note in notes().unwrap().unwrap() {
                Note { note }
            }
        }
    }
}

#[component]
fn Note(note: NoteStruct) -> Element {
    let snippets = note.snippets;

    match note.id_ {
        Some(id) => rsx! {
            p { "Note: {id}" }

            NoteSnippets { snippets }
        },
        _ => rsx! {
            NoteSnippets { snippets }
        },
    }
}

#[component]
fn NoteSnippets(snippets: Vec<SnippetStruct>) -> Element {
    rsx! {
        div {
            for snippet in snippets {
                Snippet { snippet }
            }
        }
    }
}

#[component]
fn Snippet(snippet: SnippetStruct) -> Element {
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

#[component]
fn SnippetData(data: SnippetDataEnum) -> Element {
    match data {
        SnippetDataEnum::AudioSnippet(audio_snippet) => rsx! {
            p { "Audio: {audio_snippet.path}"}
        },
        SnippetDataEnum::ImageSnippet(image_snippet) => rsx! {
            p { "Image: {image_snippet.path}"}
        },
        SnippetDataEnum::TextSnippet(text_snippet) => rsx! {
            p { "Text: {text_snippet.content}"}
        },
        SnippetDataEnum::VideoSnippet(video_snippet) => rsx! {
            p { "Video: {video_snippet.path}"}
        },
    }
}
