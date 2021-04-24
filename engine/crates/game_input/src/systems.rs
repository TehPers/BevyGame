use crate::{ActionInput, CursorState, InputBindings};
use game_camera::{CameraState, ProjectionExt, ScaledOrthographicProjection};
use game_lib::{
    bevy::{
        input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
        prelude::*,
        render::camera::Camera,
    },
    tracing::{self, instrument},
};

#[instrument(skip(actions))]
pub fn prepare_input(mut actions: ResMut<Input<ActionInput>>) {
    actions.clear();
}

#[instrument(skip(keys, input_bindings, actions))]
pub fn key_input(
    mut keys: EventReader<KeyboardInput>,
    input_bindings: Res<InputBindings>,
    mut actions: ResMut<Input<ActionInput>>,
) {
    for event in keys.iter() {
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

#[instrument(skip(mouse, input_bindings, actions))]
pub fn mouse_input(
    mut mouse: EventReader<MouseButtonInput>,
    input_bindings: Res<InputBindings>,
    mut actions: ResMut<Input<ActionInput>>,
) {
    for event in mouse.iter() {
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
    new_screen_position: Option<Vec2>,
}

#[instrument(skip(update_cursor_event, query))]
pub fn camera_changed(
    mut update_cursor_event: EventWriter<UpdateCursorEvent>,
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
            new_screen_position: None,
        });
    }
}

#[instrument(skip(cursor_moved, update_cursor_event))]
pub fn cursor_moved(
    mut cursor_moved: EventReader<CursorMoved>,
    mut update_cursor_event: EventWriter<UpdateCursorEvent>,
) {
    for event in cursor_moved.iter() {
        update_cursor_event.send(UpdateCursorEvent {
            new_screen_position: Some(event.position),
        });
    }
}

#[instrument(skip(update_cursor, cursor_state, camera_state, windows, camera_query))]
pub fn track_cursor(
    mut update_cursor: EventReader<UpdateCursorEvent>,
    mut cursor_state: ResMut<CursorState>,
    camera_state: Res<CameraState>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &ScaledOrthographicProjection, &Transform)>,
) {
    if let Ok((camera, projection, transform)) = camera_query.get(camera_state.main_camera) {
        if let Some(window) = windows.get(camera.window) {
            let (do_update, new_screen_position) =
                update_cursor.iter().fold((false, None), |(_, acc), cur| {
                    (true, cur.new_screen_position.or(acc))
                });

            if let Some(new_screen_position) = new_screen_position {
                cursor_state.screen_position = new_screen_position;
            }

            if do_update {
                let screen_size = Vec2::new(window.width(), window.height());
                cursor_state.world_position = projection
                    .screen_to_world(transform, cursor_state.screen_position, screen_size)
                    .0
                    .into();
            }
        }
    }
}
