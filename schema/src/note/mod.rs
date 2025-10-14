mod audio;
mod embedding_model_wasm32;
mod text;
mod video;

use {
    crate::SurrealRecord,
    derive::PutId,
    serde::{Deserialize, Serialize},
};

pub use text::*;

#[cfg(feature = "server")]
use surrealdb::RecordId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SnippetData {
    #[serde(rename = "audio_snippet")]
    AudioSnippet(AudioSnippet),
    #[serde(rename = "image_snippet")]
    ImageSnippet(ImageSnippet),
    #[serde(rename = "text_snippet")]
    TextSnippet(TextSnippet),
    #[serde(rename = "video_snippet")]
    VideoSnippet(VideoSnippet),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, PutId)]
pub struct Snippet {
    #[cfg(feature = "server")]
    pub id: RecordId,
    pub id_: Option<String>,
    #[serde(flatten)]
    pub data: SnippetData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSnippet {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageSnippet {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSnippet {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSnippet {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, PutId)]
pub struct Note {
    #[cfg(feature = "server")]
    pub id: RecordId,
    pub id_: Option<String>,
    #[put_ids]
    pub snippets: Vec<Snippet>,
}
