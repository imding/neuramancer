use {
    crate::optimistic::{Inflight, InflightKind, OpId, run_optimistic_with_inflight, run_refresh},
    dioxus::{logger::tracing, prelude::*},
    schema::Knot,
    std::collections::HashMap,
};

/// A context-provided store that owns the UI's local "cache" of knots and supports
/// optimistic mutations with commit (hydration) and rollback.
///
/// This is intentionally a *scaffold* that mirrors `NotesStore`:
/// - `Signal<KnotStateStruct>` as the single source of truth
/// - shared optimistic runner for create/delete/refresh
///
/// As the app grows, you can add:
/// - selection state
/// - filtering / sorting / pagination
/// - per-entity last_op guards for concurrent updates
/// - richer optimistic create (temp id, placeholder label/intent, etc)
pub struct KnotStore {
    /// Store state (local cache + UI flags).
    pub state: Signal<KnotState>,

    /// Generic inflight op bookkeeping.
    pub inflight: Signal<HashMap<OpId, Inflight>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KnotState {
    /// Local cache rendered by the UI.
    pub items: Vec<KnotVm>,

    /// Optional store-level error for surfacing in the UI (best-effort).
    pub last_error: Option<String>,
}

/// A lightweight view-model for knots.
///
/// We keep `schema::Knot` here because it already includes everything the UI might need,
/// and it should compile on both server/web (it uses `id_: Option<String>` similarly to notes).
/// If you later find `schema::Knot` has server-only fields in some build mode, switch this
/// VM to a minimal representation (like `NoteVm` did).
#[derive(Clone, Debug, PartialEq)]
pub struct KnotVm {
    /// Server-authored knot id, when known.
    pub id: Option<String>,

    /// Stable key for rendering (even when id is not present).
    pub local_key: String,

    /// The canonical knot record when known (usually after hydration).
    pub knot: Option<Knot>,

    pub status: KnotStatus,

    pub last_op: Option<OpId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum KnotStatus {
    Saved,
    PendingCreate { op_id: OpId, temp_key: String },
    Error { message: String },
}

/// Get the `KnotStore` from context.
///
/// This will panic if the provider is not mounted above the current component.
/// Prefer mounting via `StoresProvider` at the app root.
pub fn use_knot_store() -> Signal<KnotStore> {
    use_context::<Signal<KnotStore>>()
}

/// Provider component that creates the store once and hydrates it by fetching knots.
#[component]
pub fn KnotStoreProvider(children: Element) -> Element {
    let store: Signal<KnotStore> = use_signal(|| KnotStore {
        state: Signal::new(KnotState::default()),
        inflight: Signal::new(HashMap::new()),
    });

    use_context_provider(|| store);

    // Hydrate once on mount.
    use_future(move || {
        let store = store;
        async move {
            // Copy signal handles out synchronously to avoid holding a read guard across `.await`.
            let (state, inflight) = {
                let s = store.read();
                (s.state, s.inflight)
            };

            let tmp = KnotStore { state, inflight };
            tmp.refresh();
        }
    });

    children
}

impl KnotStore {
    /// Refresh the local cache from the server.
    ///
    /// This is modeled using the shared optimistic runner so it integrates with `inflight`.
    pub fn refresh(&self) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        run_refresh(
            state,
            inflight,
            || async { backend::read_knots().await },
            |s, _op_id, server_knots| {
                // Preserve any still-pending creates so they remain visible during refresh.
                let pending: Vec<KnotVm> = s
                    .items
                    .iter()
                    .filter(|vm| matches!(vm.status, KnotStatus::PendingCreate { .. }))
                    .cloned()
                    .collect();

                let mut vms: Vec<KnotVm> = server_knots
                    .into_iter()
                    .map(|knot| {
                        let id = knot.id_.clone();
                        KnotVm {
                            id: id.clone(),
                            local_key: id
                                .as_deref()
                                .map(|id| format!("knot:{id}"))
                                .unwrap_or_else(|| "knot:unknown".to_string()),
                            knot: Some(knot),
                            status: KnotStatus::Saved,
                            last_op: None,
                        }
                    })
                    .collect();

                vms.extend(pending);

                s.items = vms;
                s.last_error = None;
            },
            |s, _op_id, err| {
                let msg = format!("Failed to refresh knots: {err}");
                tracing::error!("{msg}");
                s.last_error = Some(msg);
            },
        )
    }

    /// Optimistically create a knot.
    ///
    /// Current backend API returns a full `schema::Knot` on success, so we can hydrate directly
    /// in the commit step without requiring a full refresh.
    ///
    /// This scaffold chooses a conservative approach:
    /// - optimistic: insert a pending item with temp key and no canonical knot yet
    /// - success: remove pending item and optionally refresh OR merge the returned knot
    /// - failure: remove pending item, record error
    ///
    /// You can adjust the commit policy later without changing the underlying mechanism.
    pub fn create_knot_optimistic(
        &self,
        label: String,
        intent: String,
        note_ids: Vec<String>,
        knot_ids: Vec<String>,
    ) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        // Capture only signals for use inside 'static closures.
        let refresh_state = self.state;
        let refresh_inflight = self.inflight;

        run_optimistic_with_inflight(
            state,
            inflight,
            InflightKind::Create,
            move || async move { backend::create_knot(label, intent, note_ids, knot_ids).await },
            |s, op_id| {
                let temp_key = format!("temp:{op_id}");
                let vm = KnotVm {
                    id: None,
                    local_key: format!("knot:{temp_key}"),
                    knot: None,
                    status: KnotStatus::PendingCreate {
                        op_id,
                        temp_key: temp_key.clone(),
                    },
                    last_op: Some(op_id),
                };

                s.items.insert(0, vm);
                s.last_error = None;

                CreateUndo { temp_key }
            },
            move |s, op_id, created_knot: Knot| {
                // Commit: remove pending and insert/merge created knot.
                remove_pending_create(s, op_id);

                let id = created_knot.id_.clone();
                let vm = KnotVm {
                    id: id.clone(),
                    local_key: id
                        .as_deref()
                        .map(|id| format!("knot:{id}"))
                        .unwrap_or("knot:unknown".to_string()),
                    knot: Some(created_knot),
                    status: KnotStatus::Saved,
                    last_op: Some(op_id),
                };

                // Insert at front so it appears immediately.
                s.items.insert(0, vm);
                s.last_error = None;

                // Optionally refresh for full consistency (e.g. if server normalizes/links data).
                // This is a policy decision; leaving it enabled is safe but may be redundant.
                spawn(async move {
                    let tmp = KnotStore {
                        state: refresh_state,
                        inflight: refresh_inflight,
                    };
                    tmp.refresh();
                });
            },
            |s, op_id, undo: CreateUndo, err| {
                // Rollback: remove pending item by temp key (best-effort).
                s.items
                    .retain(|vm| vm.local_key != format!("knot:{}", undo.temp_key));

                let msg = format!("Failed to create knot (op={op_id}): {err}");
                tracing::error!("{msg}");
                s.last_error = Some(msg);
            },
        )
    }

    /// Optimistically delete a knot by its server id.
    ///
    /// - optimistic: remove item immediately
    /// - success: no-op (or refresh if you expect cascading effects)
    /// - failure: reinsert removed item and mark as error
    pub fn delete_knot_optimistic(&self, id: String, recursive: bool) -> OpId {
        let state = self.state;
        let inflight = self.inflight;

        let id_for_optimistic = id.clone();

        run_optimistic_with_inflight(
            state,
            inflight,
            InflightKind::Delete,
            {
                let id = id.clone();
                move || async move { backend::delete_knot(id, recursive).await }
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
                    vm.status = KnotStatus::Error {
                        message: format!("Delete failed: {err}"),
                    };
                    vm.last_op = Some(undo.op_id);

                    let idx = index.min(s.items.len());
                    s.items.insert(idx, vm);
                }

                let msg = format!("Failed to delete knot: {err}");
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
    removed: Option<(usize, KnotVm)>,
}

fn remove_pending_create(state: &mut KnotState, op_id: OpId) {
    if let Some(pos) = state.items.iter().position(|vm| match vm.status {
        KnotStatus::PendingCreate { op_id: vm_op, .. } => vm_op == op_id,
        _ => false,
    }) {
        state.items.remove(pos);
    }
}

fn remove_by_id(state: &mut KnotState, id: &str) -> Option<(usize, KnotVm)> {
    let idx = state
        .items
        .iter()
        .position(|vm| vm.id.as_deref() == Some(id))?;
    let vm = state.items.remove(idx);
    Some((idx, vm))
}
