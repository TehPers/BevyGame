use crate::{
    camera::{ProjectionExt, ScaledOrthographicProjection},
    controller::Player,
    input::CursorState,
    physics::PhysicsBundle,
    plugins::{config::DebugConfig, timed::Timed},
};
use game_camera::CameraPlugin;
use game_controller::ControllerSystem;
use game_core::{modes::ModeExt, GameStage, GlobalMode, ModeEvent};
use game_input::InputBindings;
use game_lib::{
    bevy::{
        diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
        ecs as bevy_ecs,
        prelude::*,
        render::camera::Camera,
    },
    tracing::{self, instrument},
};
use game_physics::{PhysicsPlugin, Velocity};
use game_tiles::{EntityWorldPosition, EntityWorldRect, RegionWorldPosition};
use std::{fmt::Write, time::Duration, writeln};

struct DebugText;

#[derive(Clone, Debug)]
struct Styles {
    normal: TextStyle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct DebugPlugin;

impl DebugPlugin {
    #[instrument(skip(commands, asset_server))]
    fn setup_debug_text(mut commands: Commands, asset_server: Res<AssetServer>) {
        let fonts = Styles {
            normal: TextStyle {
                font: asset_server.load("fonts/selawik/selawk.ttf"),
                font_size: 18.0,
                color: Color::WHITE,
            },
        };

        commands
            .spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position: Rect {
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::default(),
                ..Default::default()
            })
            .insert(DebugText);

        commands.insert_resource(fonts);
    }

    #[instrument(skip(
        diagnostics,
        cursor_state,
        windows,
        input_config,
        styles,
        camera_query,
        player_query,
        text_query,
        visible_regions
    ))]
    fn update_debug_text(
        diagnostics: Res<Diagnostics>,
        cursor_state: Res<CursorState>,
        windows: Res<Windows>,
        input_config: Res<InputBindings>,
        styles: Res<Styles>,
        camera_query: Query<(&ScaledOrthographicProjection, &Camera, &Transform)>,
        player_query: Query<&Velocity, With<Player>>,
        mut text_query: Query<&mut Text, With<DebugText>>,
        visible_regions: Query<&RegionWorldPosition>,
    ) {
        let mut new_text = String::with_capacity(1000);

        // FPS
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let (Some(average), Some(duration)) = (fps.average(), fps.duration()) {
                writeln!(new_text, "FPS: {:.0} ({}ms)", average, duration.as_millis()).unwrap();
            } else {
                writeln!(new_text, "FPS: ???").unwrap();
            }
        }

        // Cursor info
        writeln!(
            new_text,
            "Cursor screen position: {}",
            cursor_state.screen_position
        )
        .unwrap();
        writeln!(
            new_text,
            "Cursor world position: {}",
            cursor_state.world_position
        )
        .unwrap();

        // Camera info
        for (projection, camera, transform) in camera_query.iter() {
            if let Some(window) = windows.get(camera.window) {
                let screen_size = Vec2::new(window.width(), window.height());
                let screen_top_left = Vec2::new(0.0, screen_size.y);
                let screen_bottom_right = Vec2::new(screen_size.x, 0.0);
                let world_top_left: Vec2 = projection
                    .screen_to_world(transform, screen_top_left, screen_size)
                    .0
                    .into();
                let world_bottom_right: Vec2 = projection
                    .screen_to_world(transform, screen_bottom_right, screen_size)
                    .0
                    .into();
                writeln!(
                    new_text,
                    "Camera {:?} world position: left: {}, right: {}, top: {}, bottom: {}",
                    camera.name.as_ref().unwrap_or(&"unnamed".into()),
                    world_top_left.x,
                    world_bottom_right.x,
                    world_top_left.y,
                    world_bottom_right.y
                )
                .unwrap();
            }
        }

        writeln!(
            new_text,
            "Rendered regions: {}",
            visible_regions.iter().count()
        )
        .unwrap();

        // Player info
        for velocity in player_query.iter() {
            writeln!(new_text, "Player velocity: {}", velocity.0).unwrap();
        }

        // Controls
        writeln!(new_text, "Controls:").unwrap();
        for (input, action) in input_config.keyboard.iter() {
            writeln!(new_text, "[{:?}]: {:?}", input, action).unwrap();
        }
        for (input, action) in input_config.mouse.iter() {
            writeln!(new_text, "[{:?}]: {:?}", input, action).unwrap();
        }

        // Update text
        let new_text = vec![TextSection {
            value: new_text,
            style: styles.normal.clone(),
        }];
        for mut text in text_query.iter_mut() {
            text.sections.clone_from(&new_text);
        }
    }

    #[instrument(skip(config, input))]
    fn debug_input(mut config: ResMut<DebugConfig>, input: Res<Input<KeyCode>>) {
        if input.just_released(KeyCode::F1) {
            config.enable_teleporting = !config.enable_teleporting;
        }
    }

    #[instrument(skip(config, input, cursor_state, player_query))]
    fn teleport_on_click(
        config: Res<DebugConfig>,
        input: Res<Input<MouseButton>>,
        cursor_state: Res<CursorState>,
        mut player_query: Query<(&mut EntityWorldRect, &mut Velocity), With<Player>>,
    ) {
        if config.enable_teleporting && input.pressed(MouseButton::Left) {
            for (mut bounds, mut velocity) in player_query.iter_mut() {
                *bounds = EntityWorldRect::from_center(
                    cursor_state.world_position.into(),
                    bounds.size() / 2.0,
                );
                velocity.0 = EntityWorldPosition::ZERO;
            }
        }
    }

    #[instrument(skip(commands, materials, input, cursor_state))]
    fn spawn_on_click(
        mut commands: Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
        input: Res<Input<MouseButton>>,
        cursor_state: Res<CursorState>,
    ) {
        if input.pressed(MouseButton::Right) {
            let size = Vec2::new(0.1, 0.1);
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        size,
                        ..Default::default()
                    },
                    transform: Transform::from_translation(cursor_state.world_position.extend(0.0)),
                    material: materials.add(ColorMaterial::color(Color::WHITE)),
                    ..Default::default()
                })
                .insert_bundle(PhysicsBundle {
                    bounds: EntityWorldRect::from_center(
                        cursor_state.world_position.into(),
                        (size / 2.0).into(),
                    ),
                    ..Default::default()
                })
                .insert(Timed::new(Duration::from_secs_f32(3.0)));
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            GameStage::GamePreUpdate,
            SystemSet::new()
                .label(DebugPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Enter))
                .with_system(Self::setup_debug_text.system()),
        )
        .add_system_set_to_stage(
            GameStage::GameUpdate,
            SystemSet::new()
                .label(DebugPlugin)
                .label(DebugSystem::ProcessInput)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                .with_system(Self::debug_input.system()),
        )
        .add_system_set_to_stage(
            GameStage::GameUpdate,
            SystemSet::new()
                .label(DebugPlugin)
                .label(DebugSystem::HandleInput)
                .before(PhysicsPlugin)
                .after(ControllerSystem::HandleControls)
                .after(DebugSystem::ProcessInput)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                .with_system(Self::teleport_on_click.system())
                .with_system(Self::spawn_on_click.system()),
        )
        .add_system_set_to_stage(
            GameStage::GameUpdate,
            SystemSet::new()
                .label(DebugPlugin)
                .label(DebugSystem::ShowDebugInfo)
                .after(PhysicsPlugin)
                .after(DebugSystem::HandleInput)
                .after(ControllerSystem::UpdateCamera)
                .after(CameraPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Active))
                // TODO: this is slow af
                // .with_system(Self::update_debug_text.system()),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub enum DebugSystem {
    ProcessInput,
    HandleInput,
    ShowDebugInfo,
}
