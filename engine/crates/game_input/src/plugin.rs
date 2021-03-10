use crate::{systems::UpdateCursorEvent, ActionInput, CursorState, InputBindings};
use bevy::prelude::*;

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
            .add_system_to_stage(stage::EVENT, crate::systems::prepare_input.system())
            .add_system_to_stage(stage::EVENT, crate::systems::key_input.system())
            .add_system_to_stage(stage::EVENT, crate::systems::mouse_input.system())
            .add_system_to_stage(stage::EVENT, crate::systems::reset_event.system())
            .add_system_to_stage(stage::EVENT, crate::systems::cursor_moved.system())
            .add_system_to_stage(stage::EVENT, crate::systems::camera_changed.system())
            .add_system_to_stage(stage::EVENT, crate::systems::track_cursor.system());
    }
}
