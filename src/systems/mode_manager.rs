use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::{components::EditingHandle, resources::LastHoveredTriggerArea};

#[derive(Debug)]
pub enum DrawingCurveMode {
    AddPointsToEnd,
    AddPointsToBeginning,
}

#[derive(Debug)]
pub enum ActiveCurve {
    Last,
    Index(usize),
}

#[derive(Debug)]
pub enum Mode {
    None,
    StartNewCurve,
    DrawingCurve(ActiveCurve, DrawingCurveMode),
    #[allow(dead_code)]
    EditingCurve(ActiveCurve),
}

impl Default for Mode {
    fn default() -> Self {
        Mode::None
    }
}

pub fn mode_manager(
    mut mode: ResMut<Mode>,
    last_hovered: Res<LastHoveredTriggerArea>, //editing handle
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&EditingHandle>,
) {
    match *mode {
        Mode::None => {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                // Are we hovering over a trigger area?
                // check if this trigger entity is an editing handle
                if let Some(editing_handle) = last_hovered
                    .0
                    .and_then(|trigger_entity| query.get(trigger_entity).ok())
                {
                    let drawing_mode = match editing_handle.handle_type {
                        crate::components::EditingHandleType::StartOfCurve => {
                            DrawingCurveMode::AddPointsToBeginning
                        }
                        crate::components::EditingHandleType::EndOfCurve => {
                            DrawingCurveMode::AddPointsToEnd
                        }
                    };

                    // Continue the curve that this editing handle belongs to
                    *mode = Mode::DrawingCurve(
                        ActiveCurve::Index(editing_handle.parent_curve),
                        drawing_mode,
                    );
                } else {
                    *mode = Mode::StartNewCurve;
                }
            }
        }
        Mode::StartNewCurve => {
            // continue the curve we just started
            *mode = Mode::DrawingCurve(ActiveCurve::Last, DrawingCurveMode::AddPointsToEnd);
        }
        Mode::DrawingCurve(..) => {
            if mouse_button_input.just_released(MouseButton::Left) {
                *mode = Mode::None;
            }
        }
        Mode::EditingCurve(_) => todo!(),
    };
}
