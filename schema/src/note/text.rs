use {
    cfg_if::cfg_if,
    chrono::{DateTime, Utc},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::FromRow;

cfg_if! {
     if #[cfg(target_arch = "wasm32")] {
         use {super::embedding_model_wasm32::EmbeddingModel};
     }
     else {
         use fastembed::EmbeddingModel;
     }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "server", derive(FromRow))]
pub struct TextNote {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoteEmbedding {
    pub note_id: String,
    pub vector: Vec<f32>,
    #[cfg_attr(feature = "serde", serde(with = "embedding_model_serde"))]
    pub model: EmbeddingModel,
}

#[cfg(feature = "serde")]
mod embedding_model_serde {
    use {
        super::*,
        serde::{Deserializer, Serializer},
        std::str::FromStr,
    };

    pub fn serialize<S>(model: &EmbeddingModel, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&model.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EmbeddingModel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EmbeddingModel::from_str(&s).map_err(serde::de::Error::custom)
    }
}
