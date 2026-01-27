use {
    crate::TestBackend,
    backend::{SnippetDataOrId, SurrealService},
    schema::{SnippetData, TextSnippet},
};

#[tokio::test]
async fn should_create_knot() {
    let mut backend = TestBackend::new().await;
    let text_snippet = SnippetData::TextSnippet(TextSnippet {
        content: "Yes".to_string(),
        embedding: Vec::new(),
    });
    let new_note = backend
        .state
        .surreal
        .create_note(vec![SnippetDataOrId::Data(text_snippet)])
        .await
        .unwrap();
    let note_id = new_note.id;
    let create_response = backend
        .state
        .surreal
        .create_knot(
            "knot 1".to_string(),
            "use it everywhere".to_string(),
            vec![note_id.clone()],
            vec![],
        )
        .await;

    assert!(create_response.is_ok());

    let read_response = backend.state.surreal.read_knots().await;

    assert!(read_response.is_ok());

    let knots = read_response.unwrap();

    assert!(knots.len() == 1);
    assert!(knots[0].notes.len() == 1);
    assert!(knots[0].knots.is_empty());
    // Verify the note ID is correct (relationship mapping)
    assert!(knots[0].notes[0].id_.as_ref().unwrap() == &note_id);

    backend.clean_up().await;
}
