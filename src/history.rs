use factory_canvas::domain::layout::FactoryLayout;

/// A point-in-time copy of the layout undo/redo needs to restore exactly.
///
/// Does NOT carry `next_entity_id` — a real design correction discovered
/// during implementation (see this feature's plan.md "Implementation
/// Deviations" for the full account). The original plan/research.md
/// Decision 6 called for storing and restoring `next_entity_id` verbatim
/// per snapshot, but that directly contradicts spec FR-009 ("the
/// identifier allocator MUST continue to move forward only... regardless
/// of any undo or redo"): restoring an older, smaller `next_entity_id`
/// after undoing a placement would let a later, unrelated placement reuse
/// the identifier the undone command had already consumed. The allocator
/// is already monotonic by construction — only `place`/`insert_into` ever
/// advance it, and nothing anywhere ever decreases it — so undo/redo
/// simply never touches it at all; every entity's own identifier is
/// already self-contained inside the restored `FactoryLayout`, so no
/// separate allocator bookkeeping is needed for a correct restore.
///
/// Carries no selection state and no viewport/camera state (research.md
/// Decision 5) — those are ephemeral editor state distinct from the
/// historical layout, consistent with `docs/data-model.md`'s existing "no
/// viewport, camera, or other editor-only metadata is persisted" rule,
/// extended here to session history rather than just saved documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditorSnapshot {
    layout: FactoryLayout,
}

impl EditorSnapshot {
    pub(crate) const fn new(layout: FactoryLayout) -> Self {
        Self { layout }
    }

    pub(crate) fn into_layout(self) -> FactoryLayout {
        self.layout
    }
}

/// The current factory session's undo/redo stacks — a whole-layout
/// snapshot history (research.md Decision 1), not a per-command-type
/// reversible-action model. `FactoryLayout` is already `Clone`, so no new
/// trait is required on any existing domain type; this module has no
/// `domain/` or egui dependency at all, mirroring `SelectedSet`'s existing
/// placement one level above `domain/` (research.md Decision 2) —
/// `docs/architecture.md`'s own "Current structure and incremental
/// target" tree already reserves exactly this module.
///
/// `EditHistory` only stores and replays snapshots; it has no knowledge of
/// *which* of the six commands produced a given entry (research.md
/// Decision 3) — that decision belongs to whichever caller calls
/// `record`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct EditHistory {
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
}

impl EditHistory {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Pushes `snapshot` (the state immediately BEFORE the command that is
    /// about to execute) onto the undo stack, and discards the entire redo
    /// stack (spec FR-004). Callers must only call this on a command's
    /// success branch, never on a rejected attempt (spec.md Assumptions).
    pub(crate) fn record(&mut self, snapshot: EditorSnapshot) {
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    /// Pops the top of the undo stack. Returns `None` and changes nothing
    /// if the undo stack is empty (spec FR-007). On `Some`, pushes
    /// `current` (the state right before this undo) onto the redo stack
    /// and returns the popped snapshot for the caller to restore.
    pub(crate) fn undo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot> {
        let restored = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(restored)
    }

    /// Symmetric to `undo`: pops the top of the redo stack, pushes
    /// `current` onto the undo stack, and returns the popped snapshot.
    /// Returns `None` and changes nothing if the redo stack is empty
    /// (spec FR-007).
    pub(crate) fn redo(&mut self, current: EditorSnapshot) -> Option<EditorSnapshot> {
        let restored = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(restored)
    }

    /// Empties both stacks (spec FR-010) — called whenever a session
    /// starts or a different factory is opened.
    pub(crate) fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use factory_canvas::domain::catalog::{
        BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId,
        CatalogMetadata, CategoryId, RegionDefinition, RegionId,
    };
    use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
    use factory_canvas::domain::layout::{BlockInstance, EntityId};
    use semver::Version;

    use super::*;

    fn test_catalog() -> Catalog {
        let metadata = CatalogMetadata::new(
            CatalogId::new("test_catalog").unwrap(),
            Version::new(1, 0, 0),
            "Test Catalog",
        );
        let region = RegionDefinition::new(RegionId::new("region_one").unwrap(), "Region One");
        let base_id = BaseId::new("base_one").unwrap();
        let base = BaseDefinition::new(
            base_id.clone(),
            "Base One",
            region.id().clone(),
            GridSize::new(10, 10).unwrap(),
        );
        let buildable = BuildableDefinition::new(
            BuildableId::new("test_block").unwrap(),
            "Test Block",
            CategoryId::new("test_category").unwrap(),
            "TB",
            GridSize::new(1, 1).unwrap(),
            vec![],
        );
        Catalog::new(
            metadata,
            base_id,
            vec![region],
            vec![base],
            vec![buildable],
            vec![],
        )
        .unwrap()
    }

    fn empty_layout() -> FactoryLayout {
        let catalog = test_catalog();
        let base_id = catalog.default_base_id().clone();
        FactoryLayout::new(catalog, base_id).unwrap()
    }

    /// Builds a layout with exactly `entity_count` distinct instances, so
    /// snapshots built from different counts are provably distinguishable
    /// by `PartialEq` — not merely "both present."
    fn layout_with_entities(entity_count: u64) -> FactoryLayout {
        let mut layout = empty_layout();
        for n in 0..entity_count {
            layout
                .place(BlockInstance::new(
                    EntityId::new(n + 1),
                    BuildableId::new("test_block").unwrap(),
                    GridPoint::new(n as i32, 0),
                    Rotation::Zero,
                ))
                .unwrap();
        }
        layout
    }

    fn snapshot_with(entity_count: u64) -> EditorSnapshot {
        EditorSnapshot::new(layout_with_entities(entity_count))
    }

    #[test]
    fn record_pushes_to_undo_and_clears_redo() {
        let mut history = EditHistory::new();
        let empty = snapshot_with(0);
        let one_entity = snapshot_with(1);

        history.record(empty.clone());
        // Simulate an undo happening, so redo_stack is non-empty...
        let restored = history.undo(snapshot_with(2));
        assert_eq!(restored, Some(empty));
        assert!(history.can_redo());

        // ...then a new record (a fresh command) must discard that redo.
        history.record(one_entity);
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_on_empty_history_returns_none_and_changes_nothing() {
        let mut history = EditHistory::new();

        let restored = history.undo(snapshot_with(0));

        assert_eq!(restored, None);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn redo_on_empty_history_returns_none_and_changes_nothing() {
        let mut history = EditHistory::new();

        let restored = history.redo(snapshot_with(0));

        assert_eq!(restored, None);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_then_redo_restores_the_pre_undo_current_snapshot() {
        let mut history = EditHistory::new();
        let before = snapshot_with(0);
        let after = snapshot_with(1);

        history.record(before.clone());
        let undone = history.undo(after.clone());
        assert_eq!(undone, Some(before));

        let redone = history.redo(snapshot_with(2));
        assert_eq!(redone, Some(after));
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut history = EditHistory::new();
        history.record(snapshot_with(0));
        history.undo(snapshot_with(1));
        assert!(history.can_redo());

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn multi_step_undo_and_redo_walks_the_stack_in_order() {
        let mut history = EditHistory::new();
        let snapshots: Vec<EditorSnapshot> = (0..4).map(snapshot_with).collect();

        // Record three transitions: 0 -> 1 -> 2 -> 3 (current).
        history.record(snapshots[0].clone());
        history.record(snapshots[1].clone());
        history.record(snapshots[2].clone());
        let mut current = snapshots[3].clone();

        // Undo three times: 3 -> 2 -> 1 -> 0.
        current = history.undo(current).unwrap();
        assert_eq!(current, snapshots[2]);
        current = history.undo(current).unwrap();
        assert_eq!(current, snapshots[1]);
        current = history.undo(current).unwrap();
        assert_eq!(current, snapshots[0]);
        assert!(!history.can_undo());

        // Redo twice: 0 -> 1 -> 2.
        current = history.redo(current).unwrap();
        assert_eq!(current, snapshots[1]);
        current = history.redo(current).unwrap();
        assert_eq!(current, snapshots[2]);
        assert!(history.can_redo());
    }
}
