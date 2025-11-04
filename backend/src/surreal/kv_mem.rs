use {
    crate::SurrealService,
    dioxus::logger::tracing,
    eyre::Result,
    include_dir::{Dir, include_dir},
    schema::{Knot, Note, SnippetData, SurrealRecord},
    surrealdb::{
        Error as SurrealError, Surreal,
        engine::local::{Db, Mem},
        error::Db as DbError,
    },
    surrealdb_migrations::MigrationRunner,
};

const SURREAL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/surreal");

#[derive(Clone)]
pub struct SurrealInMemory {
    client: Surreal<Db>,
}

impl SurrealInMemory {
    pub async fn init() -> Result<Self> {
        let client = Surreal::new::<Mem>(()).await?;

        client.use_ns("development").use_db("neuramancer").await?;

        MigrationRunner::new(&client)
            .load_files(&SURREAL_DIR)
            .up()
            .await?;

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
        let mut draft = self
            .client
            .query("BEGIN")
            .query("LET $knot = CREATE knot SET label = $label, intent = $intent")
            .bind(("label", label))
            .bind(("intent", intent));

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
        }

        // Validate that all provided knot IDs exist
        for (idx, knot_id) in knot_ids.iter().enumerate() {
            draft = draft
                .query(format!("LET $existing_knot{idx} = SELECT * FROM type::thing('knot', $knot_id{idx})"))
                .bind((format!("knot_id{idx}"), knot_id.clone()))
                .query(format!("IF array::len($existing_knot{idx}) == 0 {{ THROW 'Knot not found: ' + $knot_id{idx} }}"));
        }

        // Create converge relationships for notes
        for (idx, note_id) in note_ids.iter().enumerate() {
            draft = draft
                .query(format!(
                    "LET $note_record{idx} = type::thing('note', $note_id{idx})"
                ))
                .bind((format!("note_id{idx}"), note_id.clone()))
                .query(format!("RELATE $note_record{idx}->converge->$knot"));
        }

        // Create converge relationships for knots
        for (idx, knot_id) in knot_ids.iter().enumerate() {
            draft = draft
                .query(format!(
                    "LET $knot_record{idx} = type::thing('knot', $knot_id{idx})"
                ))
                .bind((format!("knot_id{idx}"), knot_id.clone()))
                .query(format!("RELATE $knot_record{idx}->converge->$knot"));
        }

        let mut response = draft
            .query(
                "SELECT *,
                    <-converge<-note AS notes,
                    <-converge<-knot AS knots
                FROM $knot FETCH notes, knots",
            )
            .query("COMMIT")
            .await?;

        tracing::debug!("{response:?}");

        let knots: Vec<Knot> = response.take(0)?;
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
        } else {
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
