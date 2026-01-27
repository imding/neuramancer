use {
    dioxus::logger::tracing,
    qdrant_client::{
        Qdrant as QdrantClient, QdrantError,
        qdrant::{
            CreateCollectionBuilder, Distance, ScalarQuantizationBuilder, VectorParamsBuilder,
        },
    },
};

static COLLECTION: &str = "Neuramancy";

#[derive(Clone)]
pub struct Qdrant {
    pub client: QdrantClient,
}

impl Qdrant {
    pub async fn new(
        maybe_token: Option<&str>,
        maybe_url: Option<&str>,
    ) -> Result<Self, QdrantError> {
        let url = maybe_url.expect("💦 Qdrant URL should be valid");
        let client = QdrantClient::from_url(url).api_key(maybe_token).build()?;
        // let mini_lm_options =
        //     InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true);
        // let text_embed_model =
        //     TextEmbedding::try_new(mini_lm_options).expect("Failed to get text embedding model.");

        match client.collection_exists(COLLECTION).await {
            Ok(has_collection) => match has_collection {
                true => tracing::info!("Qdrant collection exits"),
                false => {
                    let builder = CreateCollectionBuilder::new(COLLECTION)
                        .vectors_config(VectorParamsBuilder::new(384, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default());

                    match client.create_collection(builder).await {
                        Ok(response) => {
                            if !response.result {
                                tracing::error!("Failed to create Qdrant collection: {response:#?}")
                            }
                        }
                        Err(e) => tracing::error!("Failed to create Qdrant collection: {e}"),
                    }
                }
            },
            Err(e) => tracing::error!("Failed to check Qdrant collection: {e}"),
        }

        Ok(Self { client })
    }
}
