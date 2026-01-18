use {
    crate::{KnotStoreProvider, NotesStoreProvider},
    dioxus::prelude::*,
};

/// Mount all UI stores in one place.
///
/// This prevents "missing context" panics and centralizes store initialization.
#[component]
pub fn StoresProvider(children: Element) -> Element {
    rsx! {
        NotesStoreProvider {
            KnotStoreProvider {
                {children}
            }
        }
    }
}
