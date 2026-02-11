use {
    crate::SurrealRecord,
    derive::PutId,
    serde::{Deserialize, Serialize},
};

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

impl SnippetData {
    /// Returns the embedding vector for this snippet, if any.
    pub fn embedding(&self) -> &[f32] {
        match self {
            SnippetData::AudioSnippet(s) => &s.embedding,
            SnippetData::ImageSnippet(s) => &s.embedding,
            SnippetData::TextSnippet(s) => &s.embedding,
            SnippetData::VideoSnippet(s) => &s.embedding,
        }
    }
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
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageSnippet {
    pub path: String,
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSnippet {
    pub content: String,
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSnippet {
    pub path: String,
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, PutId)]
pub struct Note {
    #[cfg(feature = "server")]
    pub id: RecordId,
    pub id_: Option<String>,
    #[put_ids]
    #[serde(default)]
    pub snippets: Vec<Snippet>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let json = serde_json::to_string(value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*value, back);
    }

    #[test]
    fn text_snippet_roundtrip() {
        let snippet = TextSnippet {
            content: "hello world".into(),
            embedding: vec![0.1, 0.2, 0.3],
        };
        roundtrip(&snippet);
    }

    #[test]
    fn image_snippet_roundtrip() {
        let snippet = ImageSnippet {
            path: "/img/test.png".into(),
            embedding: vec![1.0, 2.0],
        };
        roundtrip(&snippet);
    }

    #[test]
    fn audio_snippet_roundtrip() {
        let snippet = AudioSnippet {
            path: "/audio/clip.wav".into(),
            embedding: vec![],
        };
        roundtrip(&snippet);
    }

    #[test]
    fn video_snippet_roundtrip() {
        let snippet = VideoSnippet {
            path: "/video/clip.mp4".into(),
            embedding: vec![0.5],
        };
        roundtrip(&snippet);
    }

    #[test]
    fn snippet_data_untagged_text() {
        let data = SnippetData::TextSnippet(TextSnippet {
            content: "test".into(),
            embedding: vec![],
        });
        let json = serde_json::to_string(&data).expect("serialize");
        assert!(json.contains("\"content\""));
        let back: SnippetData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(data, back);
    }

    #[test]
    fn snippet_data_untagged_image_deserializes_as_audio() {
        // BUG: #[serde(untagged)] tries variants in order. AudioSnippet and
        // ImageSnippet both have {path, embedding}, so AudioSnippet (listed
        // first) always wins. This test documents the current broken behavior.
        // Fix: switch to internally tagged enum or add a discriminating field.
        let data = SnippetData::ImageSnippet(ImageSnippet {
            path: "/img/photo.jpg".into(),
            embedding: vec![1.0],
        });
        let json = serde_json::to_string(&data).expect("serialize");
        let back: SnippetData = serde_json::from_str(&json).expect("deserialize");
        // This SHOULD be ImageSnippet but currently deserializes as AudioSnippet:
        assert!(matches!(back, SnippetData::AudioSnippet(_)));
    }

    #[test]
    fn note_with_text_and_audio_snippets() {
        // TextSnippet (has `content`) and AudioSnippet (has `path`) are
        // distinguishable under untagged serde, so this round-trips correctly.
        let note = Note {
            id_: Some("note:abc".into()),
            snippets: vec![
                Snippet {
                    id_: Some("snippet:1".into()),
                    data: SnippetData::TextSnippet(TextSnippet {
                        content: "hello".into(),
                        embedding: vec![0.1],
                    }),
                },
                Snippet {
                    id_: Some("snippet:2".into()),
                    data: SnippetData::AudioSnippet(AudioSnippet {
                        path: "/audio/clip.wav".into(),
                        embedding: vec![0.2, 0.3],
                    }),
                },
            ],
        };
        roundtrip(&note);
    }

    #[test]
    fn note_empty_snippets() {
        let note = Note {
            id_: None,
            snippets: vec![],
        };
        roundtrip(&note);
    }

    #[test]
    fn embedding_defaults_to_empty() {
        // When embedding is missing from JSON, it should default to empty vec
        let json = r#"{"content": "test"}"#;
        let snippet: TextSnippet = serde_json::from_str(json).expect("deserialize");
        assert_eq!(snippet.content, "test");
        assert!(snippet.embedding.is_empty());
    }
}
