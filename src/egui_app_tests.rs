use factory_canvas::domain::base::{BaseTemplate, SecondaryLevel};
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, PlacementError};

use super::*;

#[test]
fn base_labels_use_confirmed_names_and_derived_dimensions() {
    let labels = BaseTemplate::ALL.map(base_option_label);

    assert_eq!(
        labels,
        [
            "PAC Principal · 80 × 80",
            "Sub-PAC Padrão · 30 × 30",
            "Sub-PAC Expansão I · 40 × 40",
            "Sub-PAC Expansão II · 50 × 50",
        ]
    );
}

#[test]
fn block_labels_use_catalog_names_and_footprints() {
    let labels = BlockTemplate::ALL.map(block_option_label);

    assert_eq!(
        labels,
        [
            "Poste de Xiranita · 2 × 2",
            "Unidade de Refinaria · 3 × 3",
            "Unidade de Trituração · 3 × 3",
        ]
    );
}

#[test]
fn app_starts_with_main_base_layout() {
    let app = FactoryCanvasApp::default();

    assert_eq!(app.layout.base_template(), BaseTemplate::MainCurrent);
    assert_eq!(app.layout.bounds(), GridSize::new(80, 80).unwrap());
    assert!(app.layout.is_empty());
    assert_eq!(app.selected_block, None);
}

#[test]
fn selecting_block_keeps_template_ready_for_repeated_placements() {
    let mut app = FactoryCanvasApp::default();

    app.select_block(BlockTemplate::RefineryUnit);
    assert_eq!(app.selected_block, Some(BlockTemplate::RefineryUnit));

    app.select_block(BlockTemplate::CrushingUnit);
    assert_eq!(app.selected_block, Some(BlockTemplate::CrushingUnit));
}

#[test]
fn successful_placements_use_monotonic_ids_and_allow_edge_contact() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(BlockTemplate::XiranitePowerPole);

    app.place_selected_at(GridPoint::new(0, 0));
    app.place_selected_at(GridPoint::new(2, 0));

    assert_eq!(app.layout.len(), 2);
    assert_eq!(
        app.layout
            .instance(EntityId::new(1))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(0, 0))
    );
    assert_eq!(
        app.layout
            .instance(EntityId::new(2))
            .map(|instance| instance.origin()),
        Some(GridPoint::new(2, 0))
    );
    assert_eq!(app.next_entity_id, Some(3));
    assert_eq!(app.selected_block, Some(BlockTemplate::XiranitePowerPole));
}

#[test]
fn rejected_placements_preserve_layout_and_next_id() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(BlockTemplate::XiranitePowerPole);
    app.place_selected_at(GridPoint::new(0, 0));

    app.place_selected_at(GridPoint::new(0, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::PlacementRejected(PlacementError::Collision {
            id: EntityId::new(2),
            conflicting_id: EntityId::new(1),
        })
    );

    app.place_selected_at(GridPoint::new(79, 79));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(
        app.notice,
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds {
            id: EntityId::new(2),
        })
    );
}

#[test]
fn placement_without_selection_does_not_change_layout_or_id() {
    let mut app = FactoryCanvasApp::default();

    app.place_selected_at(GridPoint::new(0, 0));

    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(app.notice, EditorNotice::SelectBlock);
}

#[test]
fn entity_id_exhaustion_never_wraps_or_mutates_layout() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(BlockTemplate::XiranitePowerPole);
    app.next_entity_id = Some(u64::MAX);

    app.place_selected_at(GridPoint::new(0, 0));
    assert!(app.layout.instance(EntityId::new(u64::MAX)).is_some());
    assert_eq!(app.next_entity_id, None);

    app.place_selected_at(GridPoint::new(2, 0));
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, None);
    assert_eq!(app.notice, EditorNotice::EntityIdsExhausted);
}

#[test]
fn notice_text_translates_editor_state_and_domain_errors() {
    assert_eq!(
        notice_text(EditorNotice::SelectBlock),
        "Selecione um bloco para começar."
    );
    assert_eq!(
        notice_text(EditorNotice::ReadyToPlace {
            template: BlockTemplate::RefineryUnit,
        }),
        "Bloco selecionado: Unidade de Refinaria. Clique no grid para posicionar."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(PlacementError::Collision {
            id: EntityId::new(4),
            conflicting_id: EntityId::new(2),
        })),
        "Posição ocupada pelo bloco #2."
    );
    assert_eq!(
        notice_text(EditorNotice::PlacementRejected(
            PlacementError::OutOfBounds {
                id: EntityId::new(4),
            }
        )),
        "O bloco não cabe nessa posição."
    );
}

#[test]
fn instance_labels_expose_painted_blocks_semantically() {
    let instance = BlockInstance::new(
        EntityId::new(7),
        BlockTemplate::RefineryUnit,
        GridPoint::new(3, 4),
        Rotation::Zero,
    );

    assert_eq!(
        instance_semantic_label(instance),
        "#7 · Unidade de Refinaria · origem (3, 4) · 3 × 3 · 0°"
    );

    let rotated = BlockInstance::new(
        EntityId::new(8),
        BlockTemplate::RefineryUnit,
        GridPoint::new(6, 2),
        Rotation::Clockwise90,
    );
    assert_eq!(
        instance_semantic_label(rotated),
        "#8 · Unidade de Refinaria · origem (6, 2) · 3 × 3 · 90°"
    );

    assert_eq!(layout_count_label(0), "Nenhum bloco posicionado");
    assert_eq!(layout_count_label(1), "1 bloco posicionado");
    assert_eq!(layout_count_label(2), "2 blocos posicionados");
}

#[test]
fn requesting_base_change_replaces_empty_layout_immediately() {
    let mut app = FactoryCanvasApp::default();
    let templates = [
        BaseTemplate::Secondary(SecondaryLevel::Standard),
        BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI),
        BaseTemplate::Secondary(SecondaryLevel::AreaExpansionII),
        BaseTemplate::MainCurrent,
    ];

    for template in templates {
        app.request_base_change(template);

        assert_eq!(app.layout.base_template(), template);
        assert_eq!(app.layout.bounds(), template.bounds());
        assert!(app.layout.is_empty());
        assert_eq!(app.pending_base_change, None);
    }
}

#[test]
fn cancelling_nonempty_base_change_preserves_complete_state() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(BlockTemplate::XiranitePowerPole);
    app.place_selected_at(GridPoint::new(4, 5));
    let notice_before_request = app.notice;
    let target = BaseTemplate::Secondary(SecondaryLevel::Standard);

    app.request_base_change(target);

    assert_eq!(app.pending_base_change, Some(target));
    assert_eq!(app.layout.base_template(), BaseTemplate::MainCurrent);
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));

    app.cancel_base_change();

    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.base_template(), BaseTemplate::MainCurrent);
    assert_eq!(app.layout.len(), 1);
    assert_eq!(app.next_entity_id, Some(2));
    assert_eq!(app.notice, notice_before_request);
}

#[test]
fn confirming_nonempty_base_change_clears_layout_and_resets_ids() {
    let mut app = FactoryCanvasApp::default();
    app.select_block(BlockTemplate::XiranitePowerPole);
    app.place_selected_at(GridPoint::new(4, 5));
    let target = BaseTemplate::Secondary(SecondaryLevel::Standard);
    app.request_base_change(target);

    app.confirm_base_change();

    assert_eq!(app.pending_base_change, None);
    assert_eq!(app.layout.base_template(), target);
    assert!(app.layout.is_empty());
    assert_eq!(app.next_entity_id, Some(1));
    assert_eq!(app.selected_block, Some(BlockTemplate::XiranitePowerPole));
    assert_eq!(app.notice, EditorNotice::BaseChanged { template: target });
}
