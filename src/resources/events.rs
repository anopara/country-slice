use crate::systems::mode_manager::BrushMode;

pub struct CurveChangedEvent {
    pub curve_index: usize,
}

pub struct CurveDeletedEvent {
    pub curve_index: usize,
}
pub struct BrushModeJustChanged {
    pub to: BrushMode,
}
