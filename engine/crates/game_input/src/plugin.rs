use crate::{systems::UpdateCursorEvent, ActionInput, CursorState, InputBindings};
use game_core::GameStage;
use game_lib::bevy::{ecs as bevy_ecs, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.register_type::<ActionInput>()
            // TODO: .register_type::<InputBindings>()
            .register_type::<CursorState>()
            .register_type::<UpdateCursorEvent>()
            .init_resource::<InputBindings>()
            .init_resource::<Input<ActionInput>>()
            .init_resource::<CursorState>()
            .add_event::<UpdateCursorEvent>()
            .add_system_set_to_stage(
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(InputPlugin)
                    .label(InputSystem::PrepareInput)
                    .with_system(crate::systems::prepare_input.system()),
            )
            .add_system_set_to_stage(
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(InputPlugin)
                    .label(InputSystem::TranslateInput)
                    .after(InputSystem::PrepareInput)
                    .in_ambiguity_set(InputSystem::TranslateInput)
                    .with_system(crate::systems::key_input.system())
                    .with_system(crate::systems::mouse_input.system()),
            )
            .add_system_set_to_stage(
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(InputPlugin)
                    .label(InputSystem::DetectCursorChange)
                    .after(InputSystem::PrepareInput)
                    .in_ambiguity_set(InputSystem::DetectCursorChange)
                    .with_system(crate::systems::camera_changed.system())
                    .with_system(crate::systems::cursor_moved.system()),
            )
            .add_system_set_to_stage(
                GameStage::GamePreUpdate,
                SystemSet::new()
                    .label(InputPlugin)
                    .label(InputSystem::TrackCursor)
                    .after(InputSystem::DetectCursorChange)
                    .with_system(crate::systems::track_cursor.system()),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel, AmbiguitySetLabel)]
pub enum InputSystem {
    PrepareInput,
    TranslateInput,
    DetectCursorChange,
    TrackCursor,
}
