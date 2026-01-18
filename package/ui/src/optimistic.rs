use {
    dioxus::prelude::*,
    std::{
        collections::HashMap,
        future::Future,
        sync::atomic::{AtomicU64, Ordering},
    },
};

/// A unique identifier for an optimistic operation.
///
/// This is intentionally lightweight (monotonic counter) and suitable for:
/// - associating inflight state with a specific mutation
/// - guarding commit/rollback against out-of-order completion
pub type OpId = u64;

static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a fresh `OpId`.
pub fn next_op_id() -> OpId {
    NEXT_OP_ID.fetch_add(1, Ordering::Relaxed)
}

/// A generic, store-agnostic optimistic operation runner.
///
/// This utility is designed for stores that keep their local cache inside a
/// `Signal<StateStruct>` (your store's "source of truth").
///
/// The runner supports the following flow:
///
/// 1) Apply an optimistic patch to `state` immediately, capturing an `Undo` value.
/// 2) Optionally mark the op as inflight (stored in `inflight`).
/// 3) Dispatch/await the server request concurrently.
/// 4) If request succeeds:
///    - apply `commit` (which may hydrate/merge/no-op)
/// 5) If request fails:
///    - apply `rollback` with the captured `Undo`
/// 6) Clear inflight entry when done.
///
/// Notes:
/// - No assumptions are made about whether commit is a no-op or a hydration.
/// - Concurrency/out-of-order completion should be handled by the store's state design.
///   A common pattern is to embed `last_op: Option<OpId>` per entity and check it inside
///   `commit`/`rollback`.
///
/// This file intentionally does not depend on any project-specific types.
pub fn run_optimistic<State, Undo, Resp, Err, ReqFut, OptFn, CommitFn, RollbackFn>(
    mut state: Signal<State>,
    inflight: Option<Signal<HashMap<OpId, Inflight>>>,
    inflight_kind: Option<InflightKind>,
    request: impl FnOnce() -> ReqFut + 'static,
    optimistic: OptFn,
    commit: CommitFn,
    rollback: RollbackFn,
) -> OpId
where
    State: 'static,
    Undo: 'static,
    Resp: 'static,
    Err: 'static,
    ReqFut: Future<Output = Result<Resp, Err>> + 'static,
    OptFn: FnOnce(&mut State, OpId) -> Undo + 'static,
    CommitFn: FnOnce(&mut State, OpId, Resp) + 'static,
    RollbackFn: FnOnce(&mut State, OpId, Undo, Err) + 'static,
{
    let op_id = next_op_id();

    // Apply optimistic patch immediately and capture undo.
    let undo = {
        let mut w = state.write();
        optimistic(&mut w, op_id)
    };

    // Mark inflight.
    if let (Some(mut inflight_signal), Some(kind)) = (inflight, inflight_kind) {
        inflight_signal.write().insert(
            op_id,
            Inflight {
                kind,
                started: op_id,
            },
        );
    }

    // Dispatch request concurrently.
    let mut state_for_task = state;
    spawn(async move {
        let result = request().await;

        match result {
            Ok(resp) => {
                let mut w = state_for_task.write();
                commit(&mut w, op_id, resp);
            }
            Err(err) => {
                let mut w = state_for_task.write();
                rollback(&mut w, op_id, undo, err);
            }
        }
        // NOTE: clearing inflight is handled in `run_optimistic_with_inflight` when requested.
        // We can't clear here without owning `inflight` in this task.
    });

    // NOTE: We intentionally do not attempt to clear inflight here.
    // If you need inflight bookkeeping, use `run_optimistic_with_inflight`, which owns
    // the inflight signal inside the spawned task and can reliably remove the entry
    // when the request completes.

    op_id
}

/// Convenience wrapper: runs an optimistic op and manages inflight bookkeeping.
///
/// This version moves the `inflight` signal into the spawned task so it can reliably clear
/// the inflight entry when the request completes.
///
/// Prefer this API for most stores.
pub fn run_optimistic_with_inflight<State, Undo, Resp, Err, ReqFut, OptFn, CommitFn, RollbackFn>(
    mut state: Signal<State>,
    mut inflight: Signal<HashMap<OpId, Inflight>>,
    inflight_kind: InflightKind,
    request: impl FnOnce() -> ReqFut + 'static,
    optimistic: OptFn,
    commit: CommitFn,
    rollback: RollbackFn,
) -> OpId
where
    State: 'static,
    Undo: 'static,
    Resp: 'static,
    Err: 'static,
    ReqFut: Future<Output = Result<Resp, Err>> + 'static,
    OptFn: FnOnce(&mut State, OpId) -> Undo + 'static,
    CommitFn: FnOnce(&mut State, OpId, Resp) + 'static,
    RollbackFn: FnOnce(&mut State, OpId, Undo, Err) + 'static,
{
    let op_id = next_op_id();

    let undo = {
        let mut w = state.write();
        optimistic(&mut w, op_id)
    };

    inflight.write().insert(
        op_id,
        Inflight {
            kind: inflight_kind,
            started: op_id,
        },
    );

    let mut state_for_task = state;
    let mut inflight_for_task = inflight;

    spawn(async move {
        let result = request().await;

        match result {
            Ok(resp) => {
                let mut w = state_for_task.write();
                commit(&mut w, op_id, resp);
            }
            Err(err) => {
                let mut w = state_for_task.write();
                rollback(&mut w, op_id, undo, err);
            }
        }

        inflight_for_task.write().remove(&op_id);
    });

    op_id
}

/// A lightweight inflight record you can store in your store's `Signal<HashMap<OpId, Inflight>>`.
///
/// You can extend this later (timestamps, retries, etc) without changing the runner API.
#[derive(Clone, Debug, PartialEq)]
pub struct Inflight {
    /// The kind/category of operation for UI affordances (spinners, labels, etc).
    pub kind: InflightKind,

    /// A monotonic marker that can be used as a cheap "timestamp" for ordering/debugging.
    /// (Currently just the `OpId` at start.)
    pub started: OpId,
}

/// Suggested categories for inflight ops.
/// Stores can either use this directly or define their own enum and store it in `State`.
#[derive(Clone, Debug, PartialEq)]
pub enum InflightKind {
    Refresh,
    Create,
    Update,
    Delete,

    /// Domain-specific string tag when you don't want a bespoke enum yet.
    Custom(&'static str),
}

/// Helper to reduce boilerplate for "refresh" style operations.
///
/// This is still modeled as an optimistic op, but the optimistic patch is typically a no-op
/// besides setting flags in `State`.
pub fn run_refresh<State, Resp, Err, ReqFut, CommitFn, RollbackFn>(
    state: Signal<State>,
    inflight: Signal<HashMap<OpId, Inflight>>,
    request: impl FnOnce() -> ReqFut + 'static,
    commit: CommitFn,
    rollback: RollbackFn,
) -> OpId
where
    State: 'static,
    Resp: 'static,
    Err: 'static,
    ReqFut: Future<Output = Result<Resp, Err>> + 'static,
    CommitFn: FnOnce(&mut State, OpId, Resp) + 'static,
    RollbackFn: FnOnce(&mut State, OpId, Err) + 'static,
{
    run_optimistic_with_inflight(
        state,
        inflight,
        InflightKind::Refresh,
        request,
        |_state, _op_id| (),
        commit,
        |state, op_id, _undo, err| rollback(state, op_id, err),
    )
}
