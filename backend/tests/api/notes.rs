use {
    crate::TestBackend,
    backend::SurrealService,
    schema::{SnippetData, TextSnippet},
};

#[tokio::test]
async fn should_create_note() {
    let mut backend = TestBackend::new().await;
    let content = "Yes";
    let snippets = vec![SnippetData::TextSnippet(TextSnippet {
        content: content.to_string(),
    })];
    let create_response = backend.state.surreal.create_note(snippets.clone()).await;

    assert!(create_response.is_ok());

    let read_response = backend.state.surreal.read_notes().await;

    assert!(read_response.is_ok());

    let notes = read_response.unwrap();

    assert!(notes.len() == 1);
    assert!(notes[0].snippets.len() == 1);
    assert!(notes[0].snippets[0].data == snippets[0]);

    backend.clean_up().await;
}
