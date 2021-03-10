use crate::{ActionInput, CursorState, InputBindings};
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
    render::camera::Camera,
};
use game_camera::{CameraState, ProjectionExt, ScaledOrthographicProjection};
use tracing::instrument;

#[instrument(skip(actions))]
pub fn prepare_input(mut actions: ResMut<Input<ActionInput>>) {
    actions.update();
}

#[instrument(skip(keys, reader, input_bindings, actions))]
pub fn key_input(
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
pub fn mouse_input(
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

#[derive(Clone, Copy, PartialEq, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct UpdateCursorEvent {
    screen_position: Vec2,
}

#[instrument(skip(update_cursor_event))]
pub fn reset_event(mut update_cursor_event: ResMut<Events<UpdateCursorEvent>>) {
    update_cursor_event.update();
}

#[instrument(skip(cursor_state, update_cursor_event, query))]
pub fn camera_changed(
    cursor_state: Res<CursorState>,
    mut update_cursor_event: ResMut<Events<UpdateCursorEvent>>,
    query: Query<
        (),
        (
            With<Camera>,
            Or<(Changed<ScaledOrthographicProjection>, Changed<Transform>)>,
        ),
    >,
) {
    for _ in query.iter() {
        update_cursor_event.send(UpdateCursorEvent {
            screen_position: cursor_state.screen_position,
        });
    }
}

#[instrument(skip(cursor_moved, cursor_moved_reader, update_cursor_event))]
pub fn cursor_moved(
    cursor_moved: Res<Events<CursorMoved>>,
    mut cursor_moved_reader: Local<EventReader<CursorMoved>>,
    mut update_cursor_event: ResMut<Events<UpdateCursorEvent>>,
) {
    for event in cursor_moved_reader.iter(&cursor_moved) {
        update_cursor_event.send(UpdateCursorEvent {
            screen_position: event.position,
        });
    }
}

#[instrument(skip(
    update_cursor,
    update_cursor_reader,
    cursor_state,
    camera_state,
    camera_query
))]
pub fn track_cursor(
    update_cursor: Res<Events<UpdateCursorEvent>>,
    mut update_cursor_reader: Local<EventReader<UpdateCursorEvent>>,
    mut cursor_state: ResMut<CursorState>,
    camera_state: Res<CameraState>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &ScaledOrthographicProjection, &Transform)>,
) {
    if let Ok((camera, projection, transform)) = camera_query.get(camera_state.main_camera) {
        if let Some(window) = windows.get(camera.window) {
            if let Some(event) = update_cursor_reader.latest(&update_cursor) {
                cursor_state.screen_position = event.screen_position;
                let screen_size = Vec2::new(window.width(), window.height());
                cursor_state.world_position = projection
                    .screen_to_world(transform, event.screen_position, screen_size)
                    .0
                    .into();
            }
        }
    }
}
