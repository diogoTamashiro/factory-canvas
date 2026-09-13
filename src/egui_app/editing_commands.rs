use super::notices::EditorNotice;
use super::FactoryCanvasApp;
use crate::selected_set::SelectionMode;
use factory_canvas::domain::blueprint::Blueprint;
use factory_canvas::domain::catalog::{BuildableId, ProductId};
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SelectedInstanceAction {
    Move(GridPoint),
    RotateClockwise,
    SetProductionTarget(Option<ProductId>),
    RequestRemoval,
    FocusSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CanvasNavigationAction {
    FrameAll,
}

pub(super) fn canvas_navigation_action_for_frame(
    home_pressed: bool,
    has_destructive_modal: bool,
) -> Option<CanvasNavigationAction> {
    home_pressed
        .then_some(CanvasNavigationAction::FrameAll)
        .filter(|_| !has_destructive_modal)
}

pub(super) fn selected_instance_action_for_frame(
    sidebar_action: Option<SelectedInstanceAction>,
    keyboard_action: Option<SelectedInstanceAction>,
) -> Option<SelectedInstanceAction> {
    sidebar_action.or(keyboard_action)
}

pub(super) fn production_target_action_for_choice(
    current: &Option<ProductId>,
    choice: Option<ProductId>,
) -> Option<SelectedInstanceAction> {
    (choice != *current).then_some(SelectedInstanceAction::SetProductionTarget(choice))
}

impl FactoryCanvasApp {
    pub(super) fn select_block(&mut self, buildable_id: BuildableId) {
        self.selected_block = Some(buildable_id.clone());
        self.armed_blueprint = None;
        self.selected.clear();
        self.notice = EditorNotice::ReadyToPlace { buildable_id };
    }

    pub(super) fn placement_buildable_for_canvas(&self) -> Option<&BuildableId> {
        if self.destructive_modal_open() {
            None
        } else {
            self.selected_block.as_ref()
        }
    }

    pub(super) fn armed_blueprint_for_canvas(&self) -> Option<&Blueprint> {
        if self.destructive_modal_open() {
            None
        } else {
            self.armed_blueprint.as_ref()
        }
    }

    pub(super) fn select_instance(&mut self, id: EntityId) {
        self.select_instance_with_mode(id, SelectionMode::Replace);
    }

    pub(super) fn refresh_selection_notice(&mut self) {
        let layout = &self.layout;
        self.selected.retain(|id| layout.instance(id).is_some());
        self.notice = match self.selected.len() {
            0 => EditorNotice::SelectBlock,
            1 => {
                let id = self
                    .selected
                    .iter()
                    .next()
                    .expect("selection length is one");
                let instance = self
                    .layout
                    .instance(id)
                    .expect("selection was reconciled with layout");
                EditorNotice::InstanceSelected {
                    id,
                    buildable_id: instance.buildable_id().clone(),
                }
            }
            count => EditorNotice::InstancesSelected { count },
        };
    }

    pub(super) fn select_instance_with_mode(&mut self, id: EntityId, mode: SelectionMode) {
        if self.layout.instance(id).is_some() {
            self.selected_block = None;
            self.selected.apply(mode, [id]);
            self.refresh_selection_notice();
        }
    }

    pub(super) fn deselect_instance(&mut self) {
        self.selected.clear();
        self.notice = EditorNotice::SelectBlock;
    }

    pub(super) fn move_selected_by(&mut self, delta: GridPoint) {
        self.refresh_selection_notice();
        let ids: Vec<_> = self.selected.iter().collect();
        if ids.is_empty() {
            self.notice = EditorNotice::SelectBlock;
            return;
        }

        let before = self.snapshot();
        match self.layout.move_instances_by(&ids, delta) {
            Ok(()) => {
                if delta != GridPoint::new(0, 0) {
                    self.history.record(before);
                    self.session.mark_dirty();
                }
                self.selected.translate_rotation_pivot(delta);
                if ids.len() == 1 {
                    let id = ids[0];
                    let origin = self
                        .layout
                        .instance(id)
                        .expect("moved selected instance remains in layout")
                        .origin();
                    self.notice = EditorNotice::InstanceMoved { id, origin };
                } else {
                    self.notice = EditorNotice::InstancesMoved { count: ids.len() };
                }
            }
            Err(error) => self.notice = EditorNotice::InstanceEditRejected(error),
        }
    }

    pub(super) fn rotate_selected_clockwise(&mut self) {
        self.refresh_selection_notice();
        let ids: Vec<_> = self.selected.iter().collect();
        if ids.is_empty() {
            self.notice = EditorNotice::SelectBlock;
            return;
        }

        let before = self.snapshot();
        let rotation_result = if ids.len() == 1 {
            let id = ids[0];
            let rotation = self
                .layout
                .instance(id)
                .expect("selected instance was reconciled with layout")
                .rotation()
                .clockwise();
            self.layout.rotate_instance(id, rotation).map(|()| None)
        } else {
            let pivot = match self.selected.rotation_pivot() {
                Some(pivot) => Ok(pivot),
                None => self
                    .layout
                    .selection_rotation_pivot(&ids)
                    .map(|pivot| pivot.expect("multiple selected instances have a rotation pivot")),
            };
            pivot.and_then(|pivot| {
                self.layout
                    .rotate_instances_clockwise_about(&ids, pivot)
                    .map(|()| Some(pivot))
            })
        };

        match rotation_result {
            Ok(None) => {
                self.history.record(before);
                self.session.mark_dirty();
                let id = ids[0];
                let rotation = self
                    .layout
                    .instance(id)
                    .expect("rotated selected instance remains in layout")
                    .rotation();
                self.notice = EditorNotice::InstanceRotated { id, rotation };
            }
            Ok(Some(pivot)) => {
                self.history.record(before);
                self.session.mark_dirty();
                self.selected.remember_rotation_pivot(pivot);
                self.notice = EditorNotice::InstancesRotated { count: ids.len() };
            }
            Err(error) => self.notice = EditorNotice::InstanceEditRejected(error),
        }
    }

    pub(super) fn apply_selected_instance_action(&mut self, action: SelectedInstanceAction) {
        match action {
            SelectedInstanceAction::Move(delta) => self.move_selected_by(delta),
            SelectedInstanceAction::RotateClockwise => self.rotate_selected_clockwise(),
            SelectedInstanceAction::SetProductionTarget(product_id) => {
                self.set_selected_production_target(product_id)
            }
            SelectedInstanceAction::RequestRemoval => self.request_selected_instance_removal(),
            SelectedInstanceAction::FocusSelection => self.canvas.focus_selection_requested = true,
        }
    }

    pub(super) fn set_selected_production_target(&mut self, product_id: Option<ProductId>) {
        self.refresh_selection_notice();
        if self.selected.len() != 1 {
            return;
        }
        let id = self
            .selected
            .iter()
            .next()
            .expect("single selection must contain one entity ID");
        let changed = self
            .layout
            .instance(id)
            .and_then(BlockInstance::production_target)
            != product_id.as_ref();

        match self.layout.set_production_target(id, product_id.clone()) {
            Ok(()) => {
                if changed {
                    self.session.mark_dirty();
                }
                self.notice = EditorNotice::ProductionTargetChanged { id, product_id };
            }
            Err(error) => self.notice = EditorNotice::ProductionTargetRejected(error),
        }
    }

    pub(super) fn apply_canvas_interaction(
        &mut self,
        interaction: crate::egui_canvas::CanvasInteraction,
    ) {
        match interaction {
            crate::egui_canvas::CanvasInteraction::Select {
                id,
                mode: SelectionMode::Replace,
            } => self.select_instance(id),
            crate::egui_canvas::CanvasInteraction::Select { id, mode } => {
                self.select_instance_with_mode(id, mode)
            }
            crate::egui_canvas::CanvasInteraction::Place(origin) => self.place_selected_at(origin),
            crate::egui_canvas::CanvasInteraction::PlaceBlueprint(origin) => {
                self.insert_armed_blueprint_at(origin)
            }
            crate::egui_canvas::CanvasInteraction::Deselect => self.deselect_instance(),
            crate::egui_canvas::CanvasInteraction::Marquee { ids, mode } => {
                self.selected_block = None;
                self.selected.apply(mode, ids);
                self.refresh_selection_notice();
            }
        }
    }

    pub(super) fn apply_canvas_navigation_action(&mut self, action: CanvasNavigationAction) {
        match action {
            CanvasNavigationAction::FrameAll => self.canvas.viewport.frame_all(),
        }
    }

    pub(super) fn request_selected_instance_removal(&mut self) {
        if self.pending_base_change.is_some() || self.pending_unsaved_action.is_some() {
            return;
        }

        let ids: Vec<_> = self
            .selected
            .iter()
            .filter(|id| self.layout.instance(*id).is_some())
            .collect();
        self.pending_instance_removal = (!ids.is_empty()).then_some(ids);
    }

    pub(super) fn cancel_instance_removal(&mut self) {
        self.pending_instance_removal = None;
    }

    pub(super) fn confirm_instance_removal(&mut self) {
        let Some(ids) = self.pending_instance_removal.take() else {
            return;
        };
        let before = self.snapshot();
        self.selected_block = None;
        let mut removed = Vec::new();
        for id in ids {
            self.selected.remove(id);
            if let Some(instance) = self.layout.remove_instance(id) {
                removed.push(instance);
            }
        }
        if !removed.is_empty() {
            self.history.record(before);
            self.session.mark_dirty();
        }

        self.notice = match removed.as_slice() {
            [] => {
                self.refresh_selection_notice();
                return;
            }
            [instance] => EditorNotice::InstanceRemoved {
                id: instance.id(),
                buildable_id: instance.buildable_id().clone(),
            },
            instances => EditorNotice::InstancesRemoved {
                count: instances.len(),
            },
        };
    }

    pub(super) fn place_selected_at(&mut self, origin: GridPoint) {
        let Some(buildable_id) = self.selected_block.clone() else {
            self.notice = EditorNotice::SelectBlock;
            return;
        };
        let Some(next_id) = self.next_entity_id else {
            self.notice = EditorNotice::EntityIdsExhausted;
            return;
        };

        let id = EntityId::new(next_id);
        let instance = BlockInstance::new(id, buildable_id.clone(), origin, Rotation::Zero);

        let before = self.snapshot();
        match self.layout.place(instance) {
            Ok(()) => {
                self.history.record(before);
                self.session.mark_dirty();
                self.next_entity_id = next_id.checked_add(1);
                self.notice = EditorNotice::Placed {
                    id,
                    buildable_id,
                    origin,
                };
            }
            Err(error) => self.notice = EditorNotice::PlacementRejected(error),
        }
    }

    /// Arms `blueprint` for insertion (research.md Decision 5 — the
    /// "armed candidate" canvas interaction, generalized from single
    /// buildables to whole blueprints). Clears any active single-buildable
    /// placement and selection, mirroring `select_block`'s existing
    /// exclusivity contract.
    pub(super) fn arm_blueprint_for_insertion(&mut self, blueprint: Blueprint) {
        self.selected_block = None;
        self.selected.clear();
        self.armed_blueprint = Some(blueprint);
        self.notice = EditorNotice::BlueprintArmedForInsertion;
    }

    /// Attempts to insert the currently armed blueprint at `insertion_point`
    /// (spec FR-001 through FR-007). A no-op — clearing nothing, mutating
    /// nothing — if no blueprint is armed, matching `place_selected_at`'s
    /// own "recover silently rather than panic" contract for an
    /// unreachable-in-normal-UI-flow state.
    pub(super) fn insert_armed_blueprint_at(&mut self, insertion_point: GridPoint) {
        let Some(blueprint) = self.armed_blueprint.clone() else {
            return;
        };
        let Some(next_id) = self.next_entity_id else {
            self.notice = EditorNotice::EntityIdsExhausted;
            return;
        };

        let before = self.snapshot();
        match blueprint.insert_into(&mut self.layout, insertion_point, next_id) {
            Ok(next_next_id) => {
                self.history.record(before);
                self.session.mark_dirty();
                self.next_entity_id = Some(next_next_id);
                self.notice = EditorNotice::BlueprintInserted {
                    node_count: blueprint.nodes().len(),
                };
            }
            Err(error) => self.notice = EditorNotice::BlueprintInsertionRejected(error),
        }
    }
}
