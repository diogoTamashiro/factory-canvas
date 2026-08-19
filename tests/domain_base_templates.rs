use factory_canvas::domain::base::{BaseKind, BaseTemplate, SecondaryLevel};
use factory_canvas::domain::geometry::GridSize;

#[test]
fn main_current_resolves_to_80_square() {
    let expected = GridSize::new(80, 80).expect("confirmed base dimensions are positive");

    assert_eq!(BaseTemplate::MainCurrent.bounds(), expected);
}

#[test]
fn main_current_has_main_kind() {
    assert_eq!(BaseTemplate::MainCurrent.kind(), BaseKind::Main);
}

#[test]
fn secondary_standard_resolves_to_30_square() {
    let expected = GridSize::new(30, 30).expect("confirmed base dimensions are positive");
    let template = BaseTemplate::Secondary(SecondaryLevel::Standard);

    assert_eq!(template.bounds(), expected);
}

#[test]
fn secondary_template_has_secondary_kind() {
    let template = BaseTemplate::Secondary(SecondaryLevel::Standard);

    assert_eq!(template.kind(), BaseKind::Secondary);
}

#[test]
fn secondary_area_expansion_i_resolves_to_40_square() {
    let expected = GridSize::new(40, 40).expect("confirmed base dimensions are positive");
    let template = BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI);

    assert_eq!(template.bounds(), expected);
}

#[test]
fn secondary_area_expansion_ii_resolves_to_50_square() {
    let expected = GridSize::new(50, 50).expect("confirmed base dimensions are positive");
    let template = BaseTemplate::Secondary(SecondaryLevel::AreaExpansionII);

    assert_eq!(template.bounds(), expected);
}

#[test]
fn all_contains_every_confirmed_template_in_selection_order() {
    assert_eq!(
        BaseTemplate::ALL,
        [
            BaseTemplate::MainCurrent,
            BaseTemplate::Secondary(SecondaryLevel::Standard),
            BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI),
            BaseTemplate::Secondary(SecondaryLevel::AreaExpansionII),
        ]
    );
}
