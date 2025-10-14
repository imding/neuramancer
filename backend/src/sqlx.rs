use {
    chrono::Utc,
    cuid2::create_id,
    schema::TextNote,
    sqlx::{Error as SqlxError, SqlitePool, query, query_as, sqlite::SqlitePoolOptions},
    std::future::Future,
};

pub trait SqlxService {
    type Error;

    fn create_text_note(
        &self,
        content: &str,
    ) -> impl Future<Output = Result<TextNote, Self::Error>> + Send;

    fn read_text_notes(&self) -> impl Future<Output = Result<Vec<TextNote>, Self::Error>> + Send;
}

#[derive(Clone)]
pub struct SqliteInMemory {
    client: SqlitePool,
}

impl SqliteInMemory {
    pub async fn new() -> Result<Self, SqlxError> {
        let client = SqlitePoolOptions::new()
            .max_connections(20)
            .connect_with("sqlite::memory:".parse()?)
            .await?;

        query(
            r#"
            create table if not exists text_note (
                id text primary key not null,
                content text not null,
                created_at text not null
            )
            "#,
        )
        .execute(&client)
        .await?;

        Ok(Self { client })
    }
}

impl SqlxService for SqliteInMemory {
    type Error = SqlxError;

    async fn create_text_note(&self, content: &str) -> Result<TextNote, Self::Error> {
        let result: TextNote = query_as(
            r#"insert into text_note (id, content, created_at) values (?, ?, ?) returning id, content, created_at;"#
        )
            .bind(create_id())
            .bind(content)
            .bind(Utc::now().to_rfc3339())
            .fetch_one(&self.client)
            .await?;

        Ok(result)
    }

    async fn read_text_notes(&self) -> Result<Vec<TextNote>, Self::Error> {
        let results: Vec<TextNote> =
            query_as(r#"select id, content, created_at from text_note order by created_at desc;"#)
                .fetch_all(&self.client)
                .await?;

        Ok(results)
    }
}
