use {
    crate::SurrealService,
    eyre::Result,
    schema::{Note, SnippetData, SurrealRecord},
    surrealdb::{
        Error as SurrealError, Surreal,
        engine::local::{Db, Mem},
        error::Db as DbError,
    },
    surrealdb_migrations::MigrationRunner,
};

#[derive(Clone)]
pub struct SurrealInMemory {
    client: Surreal<Db>,
}

impl SurrealInMemory {
    pub async fn init() -> Result<Self> {
        let client = Surreal::new::<Mem>(()).await?;

        client.use_ns("development").use_db("neuramancer").await?;

        MigrationRunner::new(&client).up().await?;

        Ok(Self { client })
    }
}

impl SurrealService for SurrealInMemory {
    type Error = SurrealError;

    async fn create_note(&self, snippets: Vec<SnippetData>) -> Result<Note, Self::Error> {
        let mut draft = self.client.query("BEGIN").query("LET $note = CREATE note");

        for (idx, snippet) in snippets.iter().enumerate() {
            let (table, field, value) = match snippet {
                SnippetData::AudioSnippet(audio_snippet) => {
                    ("audio_snippet", "path", audio_snippet.path.as_str())
                }
                SnippetData::ImageSnippet(image_snippet) => {
                    ("image_snippet", "path", image_snippet.path.as_str())
                }
                SnippetData::TextSnippet(text_snippet) => {
                    ("text_snippet", "content", text_snippet.content.as_str())
                }
                SnippetData::VideoSnippet(video_snippet) => {
                    ("video_snippet", "path", video_snippet.path.as_str())
                }
            };

            draft = draft
                .query(format!(
                    "LET $snippet{idx} = CREATE {table} SET {field} = $value{idx}"
                ))
                .bind((format!("value{idx}"), value.to_string()))
                .query(format!("RELATE $note->contain->$snippet{idx}"));
        }

        let mut response = draft
            .query(
                "LET $result = (SELECT *, ->contain->? AS snippets FROM $note FETCH snippets)[0]",
            )
            .query("RETURN $result")
            .query("COMMIT")
            .await?;
        let maybe_note: Option<Note> = response.take(0)?;

        match maybe_note {
            Some(mut note) => {
                note.put_id();
                Ok(note)
            }
            _ => Err(SurrealError::Db(DbError::TbNotFound {
                name: "note".to_string(),
            })),
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
}
