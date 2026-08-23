use std::collections::BTreeSet;

use crate::domain::layout::EntityId;

/// Conjunto determinístico de instâncias selecionadas, ordenado por identidade
/// estável. Autoridade de bounds/colisão/footprint continua sendo `FactoryLayout`;
/// este tipo apenas agrega identidades para seleção, marquee e foco em grupo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SelectedSet {
    ids: BTreeSet<EntityId>,
}

impl SelectedSet {
    pub(crate) fn new() -> Self {
        Self {
            ids: BTreeSet::new(),
        }
    }

    pub(crate) fn contains(&self, id: EntityId) -> bool {
        self.ids.contains(&id)
    }

    pub(crate) fn insert(&mut self, id: EntityId) {
        self.ids.insert(id);
    }

    pub(crate) fn remove(&mut self, id: EntityId) {
        self.ids.remove(&id);
    }

    /// Alterna a presença de `id`: insere se ausente, remove se presente.
    pub(crate) fn toggle(&mut self, id: EntityId) {
        if self.ids.contains(&id) {
            self.ids.remove(&id);
        } else {
            self.ids.insert(id);
        }
    }

    /// União com outro conjunto (usado por marquee aditivo).
    pub(crate) fn extend(&mut self, other: impl IntoIterator<Item = EntityId>) {
        self.ids.extend(other);
    }

    /// Mantém apenas os ids que satisfazem `f` (usado por marquee de substituição
    /// e subtração: filtra instâncias cuja origem pertence ao retângulo).
    pub(crate) fn retain(&mut self, mut f: impl FnMut(EntityId) -> bool) {
        self.ids.retain(|id| f(*id));
    }

    pub(crate) fn clear(&mut self) {
        self.ids.clear();
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
    use crate::domain::layout::EntityId;

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
}
