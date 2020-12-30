use crate::{
    plugins::input::ActionInput,
    plugins::{
        camera::{CameraState, ScaledOrthographicProjection},
        config::InputBindings,
        input::CursorState,
    },
    util::ProjectionExt,
};
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
    render::camera::Camera,
};
use tracing::instrument;

pub struct InputPlugin;

impl InputPlugin {
    #[instrument(skip(actions))]
    fn prepare_input(mut actions: ResMut<Input<ActionInput>>) {
        actions.update();
    }

    #[instrument(skip(keys, reader, input_bindings, actions))]
    fn key_input(
        keys: Res<Events<KeyboardInput>>,
        mut reader: Local<EventReader<KeyboardInput>>,
        input_bindings: Res<InputBindings>,
        mut actions: ResMut<Input<ActionInput>>,
    ) {
        for event in reader.iter(&keys) {
            let key_code = match event.key_code {
                Some(key_code) => key_code,
                None => continue,
            };

            if let Some(&action) = input_bindings.keyboard.get(&key_code) {
                match event.state {
                    ElementState::Pressed => actions.press(action),
                    ElementState::Released => actions.release(action),
                }
            }
        }
    }

    #[instrument(skip(mouse, reader, input_bindings, actions))]
    fn mouse_input(
        mouse: Res<Events<MouseButtonInput>>,
        mut reader: Local<EventReader<MouseButtonInput>>,
        input_bindings: Res<InputBindings>,
        mut actions: ResMut<Input<ActionInput>>,
    ) {
        for event in reader.iter(&mouse) {
            if let Some(&action) = input_bindings.mouse.get(&event.button) {
                match event.state {
                    ElementState::Pressed => actions.press(action),
                    ElementState::Released => actions.release(action),
                }
            }
        }
    }

    #[instrument(skip(
        cursor_state,
        camera_state,
        cursor_moved,
        cursor_moved_reader,
        camera_query
    ))]
    fn track_cursor(
        mut cursor_state: ResMut<CursorState>,
        camera_state: Res<CameraState>,
        cursor_moved: Res<Events<CursorMoved>>,
        mut cursor_moved_reader: Local<EventReader<CursorMoved>>,
        windows: Res<Windows>,
        camera_query: Query<(&Camera, &ScaledOrthographicProjection, &Transform)>,
    ) {
        let (camera, projection, transform) = camera_query.get(camera_state.main_camera).unwrap();
        if let Some(window) = windows.get(camera.window) {
            for event in cursor_moved_reader.iter(&cursor_moved) {
                cursor_state.screen_position = event.position;
                let screen_size = Vec2::new(window.width(), window.height());
                cursor_state.world_position = projection
                    .screen_to_world(transform, event.position, screen_size)
                    .0;
            }
        }
    }
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Input<ActionInput>>()
            .init_resource::<CursorState>()
            .add_system_to_stage(stage::EVENT, Self::prepare_input.system())
            .add_system_to_stage(stage::EVENT, Self::key_input.system())
            .add_system_to_stage(stage::EVENT, Self::mouse_input.system())
            .add_system_to_stage(stage::EVENT, Self::track_cursor.system());
    }
}
