use factory_canvas::domain::geometry::{GridPoint, GridSize, GridSizeError, Rotation};

#[test]
fn grid_size_preserves_dimensions() {
    let size = GridSize::new(7, 4).expect("positive dimensions should be valid");

    assert_eq!(size.width(), 7);
    assert_eq!(size.height(), 4);
}

#[test]
fn grid_size_rejects_zero_width() {
    assert_eq!(GridSize::new(0, 4), Err(GridSizeError::ZeroWidth));
}

#[test]
fn grid_size_rejects_zero_height() {
    assert_eq!(GridSize::new(7, 0), Err(GridSizeError::ZeroHeight));
}

#[test]
fn zero_rotation_preserves_footprint() {
    let footprint = GridSize::new(7, 4).expect("positive dimensions should be valid");

    assert_eq!(Rotation::Zero.apply_to(footprint), footprint);
}

#[test]
fn quarter_turn_rotations_swap_footprint_axes() {
    let footprint = GridSize::new(7, 4).expect("positive dimensions should be valid");
    let rotated = GridSize::new(4, 7).expect("positive dimensions should be valid");

    assert_eq!(Rotation::Clockwise90.apply_to(footprint), rotated);
    assert_eq!(Rotation::Clockwise270.apply_to(footprint), rotated);
}

#[test]
fn half_turn_preserves_footprint_axes() {
    let footprint = GridSize::new(7, 4).expect("positive dimensions should be valid");

    assert_eq!(Rotation::Clockwise180.apply_to(footprint), footprint);
}

#[test]
fn clockwise_rotation_cycles_through_four_orientations() {
    assert_eq!(Rotation::Zero.clockwise(), Rotation::Clockwise90);
    assert_eq!(Rotation::Clockwise90.clockwise(), Rotation::Clockwise180);
    assert_eq!(Rotation::Clockwise180.clockwise(), Rotation::Clockwise270);
    assert_eq!(Rotation::Clockwise270.clockwise(), Rotation::Zero);
}

#[test]
fn grid_point_preserves_signed_coordinates() {
    let point = GridPoint::new(-2, 3);

    assert_eq!(point.x, -2);
    assert_eq!(point.y, 3);
}
