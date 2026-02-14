use {
    super::SnippetDataOrId,
    crate::{NewNote, SurrealService},
    dioxus::logger::tracing,
    embed_anything::{
        embeddings::embed::{EmbedImage, Embedder, EmbedderBuilder},
        file_processor::audio::audio_processor::AudioDecoderModel,
    },
    eyre::{Result, eyre},
    include_dir::{Dir, include_dir},
    schema::{
        AudioSnippet, ImageSnippet, Knot, Note, SnippetData, SurrealRecord, TextSnippet,
        VideoSnippet,
    },
    std::sync::Arc,
    tokio::sync::Mutex,
};

#[cfg(feature = "server")]
use {
    surrealdb::{
        Error as SurrealError, Surreal,
        engine::local::{Db, Mem},
        error::Api as ApiError,
        error::Db as DbError,
    },
    surrealdb_migrations::MigrationRunner,
};

const SURREAL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/surreal");

#[derive(Clone)]
pub struct SurrealInMemory {
    client: Surreal<Db>,
    embedder: Arc<Embedder>,
    audio_decoder: Arc<Mutex<AudioDecoderModel>>,
}

impl SurrealInMemory {
    pub async fn init() -> Result<Self> {
        let client = Surreal::new::<Mem>(()).await?;

        client.use_ns("development").use_db("neuramancer").await?;

        MigrationRunner::new(&client)
            .load_files(&SURREAL_DIR)
            .up()
            .await?;

        // Single cross-modal embedder: SigLIP supports both text and image
        // embedding in a shared 768-dimensional vector space.
        let embedder = Arc::new(
            EmbedderBuilder::new()
                .model_id(Some("google/siglip-base-patch16-224"))
                .revision(None)
                .token(None)
                .from_pretrained_hf()
                .map_err(|error| eyre!(error))?,
        );

        let audio_decoder = Arc::new(Mutex::new(
            AudioDecoderModel::from_pretrained(
                Some("openai/whisper-tiny.en"),
                Some("main"),
                "tiny-en",
                false,
            )
            .map_err(|error| eyre!(error))?,
        ));

        Ok(Self {
            client,
            embedder,
            audio_decoder,
        })
    }
}

impl SurrealInMemory {
    fn embedding_error(message: impl Into<String>) -> SurrealError {
        SurrealError::Api(ApiError::InternalError(message.into()))
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, SurrealError> {
        let embeddings = self
            .embedder
            .embed(&[text], Some(1), None)
            .await
            .map_err(|error| Self::embedding_error(format!("text embedding failed: {error}")))?;
        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Self::embedding_error("text embedding returned no vectors"))?;

        embedding
            .to_dense()
            .map_err(|error| Self::embedding_error(format!("text embedding format error: {error}")))
    }

    async fn embed_image(&self, path: &str) -> Result<Vec<f32>, SurrealError> {
        let embedding = self
            .embedder
            .embed_image(path, None)
            .await
            .map_err(|error| Self::embedding_error(format!("image embedding failed: {error}")))?;

        embedding.embedding.to_dense().map_err(|error| {
            Self::embedding_error(format!("image embedding format error: {error}"))
        })
    }

    async fn embed_transcripted_audio(&self, path: &str) -> Result<Vec<f32>, SurrealError> {
        let transcript = {
            let mut decoder = self.audio_decoder.lock().await;
            let segments = decoder.process_audio(path).map_err(|error| {
                Self::embedding_error(format!("audio transcription failed: {error}"))
            })?;

            segments
                .iter()
                .map(|segment| segment.dr.text.trim())
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };

        if transcript.trim().is_empty() {
            return Err(Self::embedding_error(
                "audio transcription produced empty transcript",
            ));
        }

        self.embed_text(&transcript).await
    }

    async fn ensure_embedding_for_existing_snippet(
        &self,
        table: &str,
        id: &str,
    ) -> Result<(), SurrealError> {
        match table {
            "text_snippet" => {
                let mut response = self
                    .client
                    .query("SELECT * FROM type::thing($table, $id)")
                    .bind(("table", table.to_string()))
                    .bind(("id", id.to_string()))
                    .await?;
                let record: Option<Vec<TextSnippet>> = response.take(0)?;
                let record = record.and_then(|mut items| items.pop()).ok_or_else(|| {
                    Self::embedding_error(format!("snippet not found: {table}:{id}"))
                })?;

                if record.embedding.is_empty() {
                    let embedding = self.embed_text(&record.content).await?;

                    self.client
                        .query("UPDATE type::thing($table, $id) SET embedding = $embedding")
                        .bind(("table", table.to_string()))
                        .bind(("id", id.to_string()))
                        .bind(("embedding", embedding))
                        .await?;
                }
            }
            "audio_snippet" => {
                let mut response = self
                    .client
                    .query("SELECT * FROM type::thing($table, $id)")
                    .bind(("table", table.to_string()))
                    .bind(("id", id.to_string()))
                    .await?;
                let record: Option<Vec<AudioSnippet>> = response.take(0)?;
                let record = record.and_then(|mut items| items.pop()).ok_or_else(|| {
                    Self::embedding_error(format!("snippet not found: {table}:{id}"))
                })?;
                if record.embedding.is_empty() {
                    let embedding = self.embed_transcripted_audio(&record.path).await?;
                    self.client
                        .query("UPDATE type::thing($table, $id) SET embedding = $embedding")
                        .bind(("table", table.to_string()))
                        .bind(("id", id.to_string()))
                        .bind(("embedding", embedding))
                        .await?;
                }
            }
            "image_snippet" => {
                let mut response = self
                    .client
                    .query("SELECT * FROM type::thing($table, $id)")
                    .bind(("table", table.to_string()))
                    .bind(("id", id.to_string()))
                    .await?;
                let record: Option<Vec<ImageSnippet>> = response.take(0)?;
                let record = record.and_then(|mut items| items.pop()).ok_or_else(|| {
                    Self::embedding_error(format!("snippet not found: {table}:{id}"))
                })?;

                if record.embedding.is_empty() {
                    let embedding = self.embed_image(&record.path).await?;

                    self.client
                        .query("UPDATE type::thing($table, $id) SET embedding = $embedding")
                        .bind(("table", table.to_string()))
                        .bind(("id", id.to_string()))
                        .bind(("embedding", embedding))
                        .await?;
                }
            }
            "video_snippet" => {
                let mut response = self
                    .client
                    .query("SELECT * FROM type::thing($table, $id)")
                    .bind(("table", table.to_string()))
                    .bind(("id", id.to_string()))
                    .await?;
                let record: Option<Vec<VideoSnippet>> = response.take(0)?;
                let record = record.and_then(|mut items| items.pop()).ok_or_else(|| {
                    Self::embedding_error(format!("snippet not found: {table}:{id}"))
                })?;

                if record.embedding.is_empty() {
                    let embedding = self.embed_transcripted_audio(&record.path).await?;

                    self.client
                        .query("UPDATE type::thing($table, $id) SET embedding = $embedding")
                        .bind(("table", table.to_string()))
                        .bind(("id", id.to_string()))
                        .bind(("embedding", embedding))
                        .await?;
                }
            }
            _ => {
                return Err(Self::embedding_error(format!(
                    "unsupported snippet table: {table}"
                )));
            }
        }

        Ok(())
    }
}

impl SurrealService for SurrealInMemory {
    type Error = SurrealError;

    async fn create_note(&self, snippets: Vec<SnippetDataOrId>) -> Result<NewNote, Self::Error> {
        for data_or_id in snippets.iter() {
            if let SnippetDataOrId::Id((table, id)) = data_or_id {
                self.ensure_embedding_for_existing_snippet(table, id)
                    .await?;
            }
        }

        let mut draft = self
            .client
            .query("BEGIN")
            .query("LET $note = CREATE ONLY note");

        for (index, data_or_id) in snippets.iter().enumerate() {
            match data_or_id {
                SnippetDataOrId::Data(data) => {
                    let (table, field, value, embedding) = match data {
                        SnippetData::AudioSnippet(audio_snippet) => {
                            let embedding = if audio_snippet.embedding.is_empty() {
                                self.embed_transcripted_audio(&audio_snippet.path).await?
                            } else {
                                audio_snippet.embedding.clone()
                            };
                            (
                                "audio_snippet",
                                "path",
                                audio_snippet.path.as_str(),
                                embedding,
                            )
                        }
                        SnippetData::ImageSnippet(image_snippet) => {
                            let embedding = if image_snippet.embedding.is_empty() {
                                self.embed_image(&image_snippet.path).await?
                            } else {
                                image_snippet.embedding.clone()
                            };
                            (
                                "image_snippet",
                                "path",
                                image_snippet.path.as_str(),
                                embedding,
                            )
                        }
                        SnippetData::TextSnippet(text_snippet) => {
                            let embedding = if text_snippet.embedding.is_empty() {
                                self.embed_text(&text_snippet.content).await?
                            } else {
                                text_snippet.embedding.clone()
                            };
                            (
                                "text_snippet",
                                "content",
                                text_snippet.content.as_str(),
                                embedding,
                            )
                        }
                        SnippetData::VideoSnippet(video_snippet) => {
                            let embedding = if video_snippet.embedding.is_empty() {
                                self.embed_transcripted_audio(&video_snippet.path).await?
                            } else {
                                video_snippet.embedding.clone()
                            };
                            (
                                "video_snippet",
                                "path",
                                video_snippet.path.as_str(),
                                embedding,
                            )
                        }
                    };

                    eprintln!("Create {table}");

                    draft = draft
                        .query(format!(
                            "LET $snippet{index} = CREATE ONLY {table} SET {field} = $value{index}, embedding = $embedding{index}"
                        ))
                        .bind((format!("value{index}"), value.to_string()))
                        .bind((format!("embedding{index}"), embedding))
                        .query(format!("RELATE $note->contain->$snippet{index}"));
                }
                SnippetDataOrId::Id((table, id)) => {
                    draft = draft
                        .query(format!(
                            "LET $existing_snippet{index} = SELECT * FROM type::thing($table{index}, $id{index})"
                        ))
                        .bind((format!("table{index}"), table.clone()))
                        .bind((format!("id{index}"), id.clone()))
                        .query(format!(
                            "IF array::len($existing_snippet{index}) == 0 {{ THROW 'Snippet not found: ' + $table{index} + ':' + $id{index} }}"
                        ))
                        .query(format!(
                            "RELATE $note->contain->type::thing($table{index}, $id{index})"
                        ))
                }
            }
        }

        let mut response = draft
            .query("LET $note_id = record::id($note.id)")
            .query("LET $snippet_ids = (SELECT VALUE record::id(out.id) FROM $note->contain)")
            .query("RETURN { id: $note_id, snippet_ids: $snippet_ids }")
            .query("COMMIT")
            .await?;
        let maybe_new_note: Option<NewNote> = response.take(0)?;

        match maybe_new_note {
            Some(new_note) => Ok(new_note),
            _ => Err(SurrealError::Db(DbError::TxFailure)),
        }
    }

    async fn read_notes(&self) -> Result<Vec<Note>, Self::Error> {
        let mut response = self
            .client
            .query("SELECT *, ->contain->? AS snippets FROM note FETCH snippets")
            .await?;
        let mut notes: Vec<Note> = response.take(0)?;

        for note in notes.iter_mut() {
            note.put_id();
        }

        Ok(notes)
    }

    async fn delete_note(&self, id: String) -> Result<(), Self::Error> {
        let mut response = self
            .client
            .query("BEGIN")
            .query("DELETE FROM contain WHERE in = type::thing('note', $id)")
            .bind(("id", id.clone()))
            .query("DELETE FROM note WHERE id = type::thing('note', $id)")
            .bind(("id", id.clone()))
            .query("COMMIT")
            .await?;

        // Check if the note was actually deleted by querying the result
        let deleted_records: Option<Vec<surrealdb::sql::Value>> = response.take(2)?;
        match deleted_records {
            Some(records) if records.is_empty() => Err(SurrealError::Db(DbError::TbNotFound {
                name: format!("note:{}", id),
            })),
            _ => Ok(()),
        }
    }

    async fn create_knot(
        &self,
        label: String,
        intent: String,
        note_ids: Vec<String>,
        knot_ids: Vec<String>,
    ) -> Result<Knot, Self::Error> {
        let mut statement_index = 0usize;
        let mut draft = self.client.query("BEGIN");

        draft = draft
            .query("LET $knot = CREATE knot SET label = $label, intent = $intent")
            .bind(("label", label))
            .bind(("intent", intent));

        statement_index += 1;

        // Validate that all provided note IDs exist
        for (idx, note_id) in note_ids.iter().enumerate() {
            draft = draft
                .query(format!(
                    "LET $note{idx} = SELECT * FROM type::thing('note', $note_id{idx})"
                ))
                .bind((format!("note_id{idx}"), note_id.clone()))
                .query(format!(
                    "IF array::len($note{idx}) == 0 {{ THROW 'Note not found: ' + $note_id{idx} }}"
                ));

            statement_index += 2;
        }

        // Validate that all provided knot IDs exist
        for (idx, knot_id) in knot_ids.iter().enumerate() {
            draft = draft
                .query(format!("LET $existing_knot{idx} = SELECT * FROM type::thing('knot', $knot_id{idx})"))
                .bind((format!("knot_id{idx}"), knot_id.clone()))
                .query(format!("IF array::len($existing_knot{idx}) == 0 {{ THROW 'Knot not found: ' + $knot_id{idx} }}"));

            statement_index += 2;
        }

        // Create converge relationships for notes
        for (idx, note_id) in note_ids.iter().enumerate() {
            draft = draft
                .query(format!(
                    "LET $note_record{idx} = type::thing('note', $note_id{idx})"
                ))
                .bind((format!("note_id{idx}"), note_id.clone()))
                .query(format!("RELATE $note_record{idx}->converge->$knot"));

            statement_index += 2;
        }

        // Create converge relationships for knots
        for (idx, knot_id) in knot_ids.iter().enumerate() {
            draft = draft
                .query(format!(
                    "LET $knot_record{idx} = type::thing('knot', $knot_id{idx})"
                ))
                .bind((format!("knot_id{idx}"), knot_id.clone()))
                .query(format!("RELATE $knot_record{idx}->converge->$knot"));

            statement_index += 2;
        }

        let select_index = statement_index;

        draft = draft.query(
            "SELECT *,
                <-converge<-note AS notes,
                <-converge<-knot AS knots
            FROM $knot FETCH notes, knots",
        );

        draft = draft.query("COMMIT");

        let mut response = draft.await?;

        tracing::debug!("{response:?}");

        let knots: Vec<Knot> = response.take(select_index)?;
        let maybe_knot = knots.into_iter().next();

        match maybe_knot {
            Some(mut knot) => {
                knot.put_id();
                Ok(knot)
            }
            _ => Err(SurrealError::Db(DbError::TbNotFound {
                name: "knot".to_string(),
            })),
        }
    }

    async fn read_knots(&self) -> Result<Vec<Knot>, Self::Error> {
        let mut response = self
            .client
            .query("SELECT *, <-converge<-note AS notes, <-converge<-knot AS knots FROM knot FETCH notes, knots")
            .await?;
        let mut knots: Vec<Knot> = response.take(0)?;

        for knot in knots.iter_mut() {
            knot.put_id();
        }

        Ok(knots)
    }

    async fn delete_knot(&self, id: String, recursive: bool) -> Result<(), Self::Error> {
        if recursive {
            // Recursive deletion: delete all notes and knots that converge into this knot
            let mut response = self
                .client
                .query("BEGIN")
                .query("LET $knot_to_delete = type::thing('knot', $id)")
                .bind(("id", id.clone()))
                .query("LET $converging_notes = SELECT VALUE in FROM converge WHERE out = $knot_to_delete AND type::is::record(in, 'note')")
                .query("LET $converging_knots = SELECT VALUE in FROM converge WHERE out = $knot_to_delete AND type::is::record(in, 'knot')")
                .query("FOR $note IN $converging_notes { DELETE FROM contain WHERE in = $note; DELETE $note }")
                .query("FOR $knot IN $converging_knots { DELETE FROM converge WHERE out = $knot; DELETE $knot }")
                .query("DELETE FROM converge WHERE out = $knot_to_delete")
                .query("DELETE $knot_to_delete")
                .query("COMMIT")
                .await?;

            // Check if the knot was actually deleted by querying the result
            let deleted_records: Option<Vec<surrealdb::sql::Value>> = response.take(7)?;
            match deleted_records {
                Some(records) if records.is_empty() => Err(SurrealError::Db(DbError::TbNotFound {
                    name: format!("knot:{id}"),
                })),
                _ => Ok(()),
            }
        }
        else {
            // Non-recursive deletion: only delete the specified knot
            let mut response = self
                .client
                .query("BEGIN")
                .query("DELETE FROM converge WHERE out = type::thing('knot', $id)")
                .bind(("id", id.clone()))
                .query("DELETE FROM knot WHERE id = type::thing('knot', $id)")
                .bind(("id", id.clone()))
                .query("COMMIT")
                .await?;

            // Check if the knot was actually deleted by querying the result
            let deleted_records: Option<Vec<surrealdb::sql::Value>> = response.take(2)?;
            match deleted_records {
                Some(records) if records.is_empty() => Err(SurrealError::Db(DbError::TbNotFound {
                    name: format!("knot:{id}"),
                })),
                _ => Ok(()),
            }
        }
    }
}
