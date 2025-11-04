mod notes;

use backend::ServerState;

pub struct TestBackend {
    pub state: ServerState,
    cleaned_up: bool,
}

impl Drop for TestBackend {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("TestBackend dropped without clean up.")
        }
    }
}

impl TestBackend {
    pub async fn new() -> Self {
        let state = ServerState::new().await;

        Self {
            state,
            cleaned_up: false,
        }
    }

    pub async fn clean_up(&mut self) {
        self.cleaned_up = true;
    }
}
