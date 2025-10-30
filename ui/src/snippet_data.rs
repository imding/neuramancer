use {dioxus::prelude::*, schema::SnippetData as SnippetDataEnum};

#[component]
pub fn SnippetData(data: SnippetDataEnum) -> Element {
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
