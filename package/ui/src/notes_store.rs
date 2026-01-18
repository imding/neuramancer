use {
    crate::optimistic::{run_optimistic_with_inflight, run_refresh, Inflight, InflightKind, OpId},
    backend::NewNote,
    dioxus::{logger::tracing, prelude::*},
    std::collections::HashMap,
};

/// A context-provided store that owns the UI's local "cache" of notes and supports
/// optimistic mutations with commit (hydration) and rollback.
///
/// This store is implemented using a single `Signal<NotesState>` so it scales as more
/// fields are added (errors, pagination, selection, etc).
///
/// Important: we intentionally avoid constructing `schema::Note` for optimistic/pending
/// items because that type has server-only fields under `schema`'s `server` feature and
/// may be compiled in different modes/targets by `dx`.
pub struct NotesStore {
    /// Store state (local cache + any UI flags).
    pub state: Signal<NotesState>,

    /// Generic inflight op bookkeeping.
    pub inflight: Signal<HashMap<OpId, Inflight>>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct NotesState {
    /// Local cache rendered by the UI.
    pub items: Vec<NoteVm>,

    /// Optional store-level error for surfacing in the UI (best-effort).
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NoteStatus {
    Saved,
    PendingCreate { op_id: OpId, temp_key: String },
    Error { message: String },
}

/// Minimal view-model for notes that is safe to construct on any target.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteVm {
    /// Server-authored note id, when known.
    pub id: Option<String>,

    /// A stable key for rendering even when `id` is not present (pending create).
    pub local_key: String,

    /// Snippet count for display.
    pub snippet_count: usize,

    pub status: NoteStatus,

    /// The last op that mutated this entity. Can be used to guard against out-of-order
    /// completion if you add concurrent updates later.
    pub last_op: Option<OpId>,
}

/// Get the `NotesStore` from context. Panics if the provider is not mounted.
pub fn use_notes_store() -> Signal<NotesStore> {
    use_context::<Signal<NotesStore>>()
}

/// Provider component that creates the store once and hydrates it by fetching notes.
///
/// Mount this high in your app tree (e.g. root App) so SSR and all routes can access it.
#[component]
pub fn NotesStoreProvider(children: Element) -> Element {
    let store: Signal<NotesStore> = use_signal(|| NotesStore {
        state: Signal::new(NotesState::default()),
        inflight: Signal::new(HashMap::new()),
    });

    use_context_provider(|| store);

    // Hydrate once on mount.
    use_future(move || {
        let store = store;
        async move {
            // Copy the signal handles out synchronously to avoid holding a read guard across `.await`.
            let (state, inflight) = {
                let s = store.read();
                (s.state, s.inflight)
            };

            let tmp = NotesStore { state, inflight };
            tmp.refresh();
        }
    });

    children
}

impl NotesStore {
    /// Refresh the local cache from the server.
    ///
    /// This is modeled using the shared optimistic runner so it integrates with `inflight`.
    pub fn refresh(&self) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        run_refresh(
            state,
            inflight,
            || async { backend::read_notes().await },
            |s, _op_id, server_notes| {
                // Preserve any still-pending creates to avoid them disappearing during refresh.
                let pending: Vec<NoteVm> = s
                    .items
                    .iter()
                    .filter(|&vm| matches!(vm.status, NoteStatus::PendingCreate { .. }))
                    .cloned()
                    .collect();

                let mut vms: Vec<NoteVm> = server_notes
                    .into_iter()
                    .map(|note| NoteVm {
                        id: note.id_.clone(),
                        local_key: note
                            .id_
                            .clone()
                            .map(|id| format!("note:{id}"))
                            .unwrap_or_else(|| "note:unknown".to_string()),
                        snippet_count: note.snippets.len(),
                        status: NoteStatus::Saved,
                        last_op: None,
                    })
                    .collect();

                vms.extend(pending);

                s.items = vms;
                s.last_error = None;
            },
            |s, _op_id, err| {
                let msg = format!("Failed to refresh notes: {err}");
                tracing::error!("{msg}");
                s.last_error = Some(msg);
            },
        )
    }

    /// Optimistically create a note.
    ///
    /// - optimistic: insert a pending item with a temp key
    /// - success: remove pending item and hydrate by refreshing (server assigns IDs)
    /// - failure: remove pending item, record error
    pub fn create_note_optimistic(&self, content: String) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        // Capture only signals (Copy) for use inside 'static closures.
        let refresh_state = self.state;
        let refresh_inflight = self.inflight;

        run_optimistic_with_inflight(
            state,
            inflight,
            InflightKind::Create,
            move || async move { backend::save_note(content).await },
            |s, op_id| {
                let temp_key = format!("temp:{op_id}");
                let vm = NoteVm {
                    id: None,
                    local_key: format!("note:{temp_key}"),
                    snippet_count: 0,
                    status: NoteStatus::PendingCreate {
                        op_id,
                        temp_key: temp_key.clone(),
                    },
                    last_op: Some(op_id),
                };

                s.items.insert(0, vm);
                s.last_error = None;

                CreateUndo { temp_key }
            },
            move |s, op_id, _resp: NewNote| {
                // Commit: remove pending item
                remove_pending_create(s, op_id);

                // Hydrate: refresh into state without capturing `self`.
                spawn(async move {
                    refresh_notes_into_state(refresh_state, refresh_inflight);
                });
            },
            |s, op_id, undo: CreateUndo, err| {
                // Rollback: remove pending item by temp key (best-effort)
                s.items
                    .retain(|vm| vm.local_key != format!("note:{}", undo.temp_key));

                let msg = format!("Failed to create note (op={op_id}): {err}");
                tracing::error!("{msg}");
                s.last_error = Some(msg);
            },
        )
    }

    /// Optimistically delete a note by its server id.
    ///
    /// - optimistic: remove item immediately
    /// - success: no-op (can hydrate later if needed)
    /// - failure: reinsert removed item and mark as error
    pub fn delete_note_optimistic(&self, id: String) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        let id_for_optimistic = id.clone();

        run_optimistic_with_inflight(
            state,
            inflight,
            InflightKind::Delete,
            {
                let id = id.clone();
                move || async move { backend::delete_note(id).await }
            },
            move |s, op_id| {
                let removed = remove_by_id(s, &id_for_optimistic);
                s.last_error = None;
                DeleteUndo { op_id, removed }
            },
            |_s, _op_id, _resp: ()| {
                // Commit: no-op by default.
                // If deletions have server-side cascading effects later, hydrate here.
            },
            |s, _op_id, undo: DeleteUndo, err| {
                if let Some((index, mut vm)) = undo.removed {
                    vm.status = NoteStatus::Error {
                        message: format!("Delete failed: {err}"),
                    };
                    vm.last_op = Some(undo.op_id);

                    let idx = index.min(s.items.len());
                    s.items.insert(idx, vm);
                }

                let msg = format!("Failed to delete note: {err}");
                tracing::error!("{msg}");
                s.last_error = Some(msg);
            },
        )
    }
}

#[derive(Clone, Debug)]
struct CreateUndo {
    temp_key: String,
}

#[derive(Clone, Debug)]
struct DeleteUndo {
    op_id: OpId,
    removed: Option<(usize, NoteVm)>,
}

fn remove_pending_create(state: &mut NotesState, op_id: OpId) {
    // Remove the first matching PendingCreate with this op_id.
    if let Some(pos) = state.items.iter().position(|vm| match vm.status {
        NoteStatus::PendingCreate { op_id: vm_op, .. } => vm_op == op_id,
        _ => false,
    }) {
        state.items.remove(pos);
    }
}

fn remove_by_id(state: &mut NotesState, id: &str) -> Option<(usize, NoteVm)> {
    let idx = state
        .items
        .iter()
        .position(|vm| vm.id.as_deref() == Some(id))?;
    let vm = state.items.remove(idx);
    Some((idx, vm))
}

/// Fire-and-forget hydration helper used by optimistic commits without capturing `self`.
///
/// This calls `backend::read_notes()` and replaces `NotesState.items` with the server result,
/// while preserving any still-pending creates so they remain visible during refresh.
fn refresh_notes_into_state(state: Signal<NotesState>, inflight: Signal<HashMap<OpId, Inflight>>) {
    // Reuse the store's existing refresh mechanism by constructing a lightweight store value.
    // This avoids capturing `&self` in a `'static` closure.
    let tmp = NotesStore { state, inflight };
    tmp.refresh();
}
