use std::collections::BTreeSet;

use factory_canvas::domain::{geometry::GridPoint, layout::EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionMode {
    Replace,
    Add,
    Toggle,
}

/// Conjunto determinístico de instâncias selecionadas, ordenado por identidade
/// estável. Autoridade de bounds/colisão/footprint continua sendo `FactoryLayout`;
/// este tipo apenas agrega identidades para seleção, marquee e foco em grupo,
/// além de lembrar o pivô transitório enquanto a composição não muda.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SelectedSet {
    ids: BTreeSet<EntityId>,
    rotation_pivot: Option<GridPoint>,
}

impl SelectedSet {
    pub(crate) fn new() -> Self {
        Self {
            ids: BTreeSet::new(),
            rotation_pivot: None,
        }
    }

    pub(crate) fn contains(&self, id: EntityId) -> bool {
        self.ids.contains(&id)
    }

    pub(crate) fn insert(&mut self, id: EntityId) {
        if self.ids.insert(id) {
            self.rotation_pivot = None;
        }
    }

    pub(crate) fn remove(&mut self, id: EntityId) {
        if self.ids.remove(&id) {
            self.rotation_pivot = None;
        }
    }

    /// Alterna a presença de `id`: insere se ausente, remove se presente.
    pub(crate) fn toggle(&mut self, id: EntityId) {
        if !self.ids.remove(&id) {
            self.ids.insert(id);
        }
        self.rotation_pivot = None;
    }

    /// União com outro conjunto (usado por marquee aditivo).
    pub(crate) fn extend(&mut self, other: impl IntoIterator<Item = EntityId>) {
        for id in other {
            self.insert(id);
        }
    }

    pub(crate) fn apply(
        &mut self,
        mode: SelectionMode,
        candidates: impl IntoIterator<Item = EntityId>,
    ) {
        let candidates: BTreeSet<_> = candidates.into_iter().collect();
        match mode {
            SelectionMode::Replace => {
                if self.ids != candidates {
                    self.ids = candidates;
                    self.rotation_pivot = None;
                }
            }
            SelectionMode::Add => self.extend(candidates),
            SelectionMode::Toggle => {
                for id in candidates {
                    self.toggle(id);
                }
            }
        }
    }

    /// Mantém apenas IDs ainda válidas no layout corrente.
    pub(crate) fn retain(&mut self, mut f: impl FnMut(EntityId) -> bool) {
        let previous_len = self.ids.len();
        self.ids.retain(|id| f(*id));
        if self.ids.len() != previous_len {
            self.rotation_pivot = None;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.ids.clear();
        self.rotation_pivot = None;
    }

    pub(crate) const fn rotation_pivot(&self) -> Option<GridPoint> {
        self.rotation_pivot
    }

    pub(crate) fn remember_rotation_pivot(&mut self, pivot: GridPoint) {
        self.rotation_pivot = (self.ids.len() > 1).then_some(pivot);
    }

    pub(crate) fn translate_rotation_pivot(&mut self, delta: GridPoint) {
        let Some(pivot) = self.rotation_pivot else {
            return;
        };
        self.rotation_pivot = pivot
            .x
            .checked_add(delta.x)
            .zip(pivot.y.checked_add(delta.y))
            .map(|(x, y)| GridPoint::new(x, y));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.ids.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.ids.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use factory_canvas::domain::geometry::GridPoint;
    use factory_canvas::domain::layout::EntityId;

    use super::*;

    fn id(value: u64) -> EntityId {
        EntityId::new(value)
    }

    #[test]
    fn new_set_is_empty() {
        let set = SelectedSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(set.iter().next().is_none());
    }

    #[test]
    fn insert_keeps_unique_and_ordered() {
        let mut set = SelectedSet::new();
        set.insert(id(3));
        set.insert(id(1));
        set.insert(id(2));
        set.insert(id(1));

        assert_eq!(set.len(), 3);
        let collected: Vec<_> = set.iter().collect();
        assert_eq!(collected, vec![id(1), id(2), id(3)]);
    }

    #[test]
    fn remove_of_absent_id_is_noop() {
        let mut set = SelectedSet::new();
        set.insert(id(5));
        set.remove(id(9));
        assert_eq!(set.len(), 1);
        assert!(set.contains(id(5)));
    }

    #[test]
    fn toggle_adds_when_absent_and_removes_when_present() {
        let mut set = SelectedSet::new();
        set.toggle(id(4));
        assert!(set.contains(id(4)));
        set.toggle(id(4));
        assert!(!set.contains(id(4)));
        assert!(set.is_empty());
    }

    #[test]
    fn extend_performs_union_without_duplicates() {
        let mut set = SelectedSet::new();
        set.insert(id(1));
        set.extend([id(1), id(2), id(3)]);
        let collected: Vec<_> = set.iter().collect();
        assert_eq!(collected, vec![id(1), id(2), id(3)]);
    }

    #[test]
    fn retain_filters_by_predicate() {
        let mut set = SelectedSet::new();
        set.extend([id(1), id(2), id(3), id(4)]);
        set.retain(|candidate| candidate.value() % 2 == 0);
        let collected: Vec<_> = set.iter().collect();
        assert_eq!(collected, vec![id(2), id(4)]);
    }

    #[test]
    fn clear_empties_the_set() {
        let mut set = SelectedSet::new();
        set.extend([id(1), id(2)]);
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn apply_selection_modes_replace_add_and_toggle_deterministically() {
        let mut set = SelectedSet::new();
        set.extend([id(1), id(2)]);

        set.apply(SelectionMode::Replace, [id(3), id(3)]);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![id(3)]);

        set.apply(SelectionMode::Add, [id(1), id(3)]);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![id(1), id(3)]);

        set.apply(SelectionMode::Toggle, [id(1), id(2), id(2)]);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![id(2), id(3)]);
    }

    #[test]
    fn selection_membership_changes_invalidate_remembered_rotation_pivot() {
        let mut set = SelectedSet::new();
        set.extend([id(1), id(2)]);

        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.insert(id(3));
        assert_eq!(set.rotation_pivot(), None);

        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.remove(id(3));
        assert_eq!(set.rotation_pivot(), None);

        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.toggle(id(2));
        assert_eq!(set.rotation_pivot(), None);

        set.insert(id(2));
        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.apply(SelectionMode::Replace, [id(2), id(3)]);
        assert_eq!(set.rotation_pivot(), None);

        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.retain(|candidate| candidate == id(2));
        assert_eq!(set.rotation_pivot(), None);

        set.remember_rotation_pivot(GridPoint::new(4, 5));
        set.clear();
        assert_eq!(set.rotation_pivot(), None);
    }

    #[test]
    fn reapplying_identical_selection_preserves_remembered_rotation_pivot() {
        let mut set = SelectedSet::new();
        let pivot = GridPoint::new(8, 9);
        set.extend([id(1), id(2)]);
        set.remember_rotation_pivot(pivot);

        set.insert(id(1));
        set.remove(id(99));
        set.extend([id(2), id(1)]);
        set.apply(SelectionMode::Replace, [id(2), id(1)]);
        set.retain(|_| true);

        assert_eq!(set.rotation_pivot(), Some(pivot));
    }

    #[test]
    fn translating_selection_moves_remembered_rotation_pivot_by_same_delta() {
        let mut set = SelectedSet::new();
        set.extend([id(1), id(2)]);
        set.remember_rotation_pivot(GridPoint::new(8, 9));

        set.translate_rotation_pivot(GridPoint::new(-3, 4));

        assert_eq!(set.rotation_pivot(), Some(GridPoint::new(5, 13)));
    }

    #[test]
    fn single_selection_does_not_remember_group_rotation_pivot() {
        let mut set = SelectedSet::new();
        set.insert(id(1));

        set.remember_rotation_pivot(GridPoint::new(8, 9));

        assert_eq!(set.rotation_pivot(), None);
    }
}
