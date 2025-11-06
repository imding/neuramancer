use {
    crate::TestBackend,
    backend::{SnippetDataOrId, SurrealService},
    schema::{SnippetData, TextSnippet},
};

#[tokio::test]
async fn should_create_note() {
    let mut backend = TestBackend::new().await;
    let content = "Yes";
    let text_snippet = SnippetData::TextSnippet(TextSnippet {
        content: content.to_string(),
    });

    match backend
        .state
        .surreal
        .create_note(vec![SnippetDataOrId::Data(text_snippet.clone())])
        .await
    {
        Ok(new_note) => {
            assert!(new_note.snippet_ids.len() == 1);

            let read_response = backend.state.surreal.read_notes().await;

            assert!(read_response.is_ok());

            let notes = read_response.unwrap();

            assert!(notes.len() == 1);
            assert!(notes[0].snippets.len() == 1);
            assert!(notes[0].snippets[0].data == text_snippet);
            assert!(notes[0].id_.as_ref().unwrap() == &new_note.id);
        }
        Err(error) => {
            eprintln!("{error}");
            assert!(false);
        }
    };

    backend.clean_up().await;
}
