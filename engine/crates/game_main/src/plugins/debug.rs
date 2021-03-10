use crate::{
    camera::{ProjectionExt, ScaledOrthographicProjection},
    controller::Player,
    input::CursorState,
    physics::{
        bodies::AxisAlignedBoundingBox, broad_phase::QuadTreeNode, BodyType, BroadPhaseQuadTree,
        PhysicsBundle,
    },
    plugins::{config::DebugConfig, timed::Timed},
};
use bevy::{
    app::startup_stage,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::camera::Camera,
};
use game_input::InputBindings;
use game_physics::Velocity;
use std::{fmt::Write, time::Duration, writeln};
use tracing::instrument;

struct QuadTreeRegion;

struct DebugText;

pub struct DebugPlugin;

impl DebugPlugin {
    #[instrument(skip(commands, asset_server))]
    fn setup_debug_text(commands: &mut Commands, asset_server: Res<AssetServer>) {
        commands
            .spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    ..Default::default()
                },
                text: Text {
                    value: "debug info".into(),
                    font: asset_server.load("fonts/selawik/selawk.ttf"),
                    style: TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::zero()),
                ..Default::default()
            })
            .with(DebugText);
    }

    #[instrument(skip(
        diagnostics,
        cursor_state,
        windows,
        input_config,
        camera_query,
        player_query,
        text_query
    ))]
    fn update_debug_text(
        diagnostics: Res<Diagnostics>,
        cursor_state: Res<CursorState>,
        windows: Res<Windows>,
        input_config: Res<InputBindings>,
        camera_query: Query<(&ScaledOrthographicProjection, &Camera, &Transform)>,
        player_query: Query<&Velocity, With<Player>>,
        mut text_query: Query<&mut Text, With<DebugText>>,
    ) {
        let mut new_text = String::new();

        // FPS
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                writeln!(new_text, "FPS: {:.0}", average).unwrap();
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

        for mut text in text_query.iter_mut() {
            text.value.clone_from(&new_text);
        }
    }

    #[instrument(skip(config, input))]
    fn debug_input(mut config: ResMut<DebugConfig>, input: Res<Input<KeyCode>>) {
        if input.just_released(KeyCode::F1) {
            config.enable_teleporting = !config.enable_teleporting;
        }
        if input.just_released(KeyCode::F2) {
            config.show_quadtree = !config.show_quadtree;
        }
    }

    #[instrument(skip(config, input, cursor_state, player_query))]
    fn teleport_on_click(
        config: Res<DebugConfig>,
        input: Res<Input<MouseButton>>,
        cursor_state: Res<CursorState>,
        mut player_query: Query<(&mut AxisAlignedBoundingBox, &mut Velocity), With<Player>>,
    ) {
        if config.enable_teleporting && input.pressed(MouseButton::Left) {
            for (mut bounds, mut velocity) in player_query.iter_mut() {
                *bounds = AxisAlignedBoundingBox::from_center(
                    cursor_state.world_position.into(),
                    bounds.size(),
                );
                velocity.0 = Vec2::zero();
            }
        }
    }

    #[instrument(skip(commands, materials, input, cursor_state))]
    fn spawn_on_click(
        commands: &mut Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
        input: Res<Input<MouseButton>>,
        cursor_state: Res<CursorState>,
    ) {
        if input.pressed(MouseButton::Right) {
            let size = Vec2::new(0.1, 0.1);
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        size,
                        ..Default::default()
                    },
                    transform: Transform::from_translation(cursor_state.world_position.extend(0.0)),
                    material: materials.add(ColorMaterial::color(Color::WHITE)),
                    ..Default::default()
                })
                .with_bundle(PhysicsBundle {
                    bounds: AxisAlignedBoundingBox::from_center(
                        cursor_state.world_position.into(),
                        size,
                    ),
                    body_type: BodyType::Static,
                    ..Default::default()
                })
                .with(Timed::new(Duration::from_secs_f32(3.0)));
        }
    }

    #[instrument(skip(commands, materials, quadtree, query))]
    fn show_quads(
        commands: &mut Commands,
        config: Res<DebugConfig>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        quadtree: ChangedRes<BroadPhaseQuadTree>,
        query: Query<Entity, With<QuadTreeRegion>>,
    ) {
        if !config.show_quadtree {
            return;
        }

        // Remove existing markers
        for entity in query.iter() {
            commands.despawn(entity);
        }

        fn create_markers<
            const MIN_ENTRIES: usize,
            const MAX_ENTRIES: usize,
            const MAX_DEPTH: usize,
        >(
            commands: &mut Commands,
            materials: &mut Assets<ColorMaterial>,
            node: &QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>,
        ) {
            match node {
                QuadTreeNode::Leaf { bounds, .. } => {
                    create_marker(commands, materials, *bounds);
                }
                QuadTreeNode::Inner {
                    bounds, children, ..
                } => {
                    create_marker(commands, materials, *bounds);
                    for child in children.iter() {
                        create_markers(commands, materials, child);
                    }
                }
            }
        }

        fn create_marker(
            commands: &mut Commands,
            materials: &mut Assets<ColorMaterial>,
            bounds: AxisAlignedBoundingBox,
        ) {
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        size: bounds.size(),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(bounds.center().extend(0.0)),
                    material: materials.add(ColorMaterial::color(*Color::WHITE.clone().set_a(0.1))),
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .with(QuadTreeRegion);
        }

        let materials = &mut *materials;
        create_markers(commands, materials, quadtree.root());
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(
            startup_stage::PRE_STARTUP,
            Self::setup_debug_text.system(),
        )
        .add_system_to_stage(stage::EVENT, Self::debug_input.system())
        .add_system_to_stage(stage::UPDATE, Self::teleport_on_click.system())
        .add_system_to_stage(stage::UPDATE, Self::spawn_on_click.system())
        .add_system_to_stage(stage::UPDATE, Self::update_debug_text.system())
        .add_stage_before(
            stage::POST_UPDATE,
            "show_quads",
            SystemStage::single(Self::show_quads.system()),
        );
    }
}
