use super::notices::EditorNotice;
use super::FactoryCanvasApp;
use crate::history::EditorSnapshot;

impl FactoryCanvasApp {
    /// Builds an `EditorSnapshot` of the current layout, for
    /// `EditHistory::record`/`undo`/`redo` (research.md Decision 3 —
    /// `EditHistory` itself has no knowledge of which command is
    /// snapshotting; each of the six mutation call sites decides when to
    /// call this). Does not capture `next_entity_id` — see
    /// `EditorSnapshot`'s own doc comment for why the allocator is never
    /// part of undo/redo history at all (FR-009).
    pub(super) fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot::new(self.layout.clone())
    }

    /// Restores `self.layout` from `restored`, reconciling everything
    /// else the same way every other layout-replacing action already
    /// does: `self.selected` is pruned (never restored — research.md
    /// Decision 5), the session is marked dirty, and `notice` is set.
    /// `self.next_entity_id` is deliberately left untouched — see
    /// `EditorSnapshot`'s doc comment (FR-009). Shared by `undo`/`redo`.
    pub(super) fn apply_restored_snapshot(
        &mut self,
        restored: EditorSnapshot,
        notice: EditorNotice,
    ) {
        self.layout = restored.into_layout();
        self.canvas.rotation_visuals.resync(&self.layout);
        self.refresh_selection_notice();
        self.session.mark_dirty();
        self.notice = notice;
    }

    /// Undoes the single most-recently executed command (spec FR-001,
    /// FR-002). A no-op while a destructive confirmation is pending
    /// (FR-008) or the undo history is empty (FR-007).
    pub(super) fn undo(&mut self) {
        if self.destructive_modal_open() {
            return;
        }
        let current = self.snapshot();
        if let Some(restored) = self.history.undo(current) {
            self.apply_restored_snapshot(restored, EditorNotice::Undone);
        }
    }

    /// Redoes the single most-recently undone command (spec FR-003).
    /// Symmetric to `undo`: a no-op while a destructive confirmation is
    /// pending (FR-008) or the redo history is empty (FR-007).
    pub(super) fn redo(&mut self) {
        if self.destructive_modal_open() {
            return;
        }
        let current = self.snapshot();
        if let Some(restored) = self.history.redo(current) {
            self.apply_restored_snapshot(restored, EditorNotice::Redone);
        }
    }
}
