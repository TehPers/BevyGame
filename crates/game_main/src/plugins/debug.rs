use crate::plugins::{
    config::DebugConfig,
    input::CursorState,
    physics::{AxisAlignedBoundingBox, BodyType, BroadPhaseQuadTree, PhysicsBundle, QuadTreeNode},
    timed::Timed,
    Player,
};
use bevy::prelude::*;
use std::time::Duration;
use tracing::instrument;

struct QuadTreeMarker;

pub struct DebugPlugin;

impl DebugPlugin {
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
        mut player_query: Query<&mut AxisAlignedBoundingBox, With<Player>>,
    ) {
        if config.enable_teleporting && input.pressed(MouseButton::Left) {
            for mut bounds in player_query.iter_mut() {
                bounds.top_left = Vec2::from(cursor_state.world_position) - bounds.size / 2.0;
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
                    transform: Transform::from_translation(cursor_state.world_position),
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
        query: Query<Entity, With<QuadTreeMarker>>,
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
                        size: bounds.size,
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
                .with(QuadTreeMarker);
        }

        let materials = &mut *materials;
        create_markers(commands, materials, &quadtree.root);
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::EVENT, Self::debug_input.system())
            .add_system_to_stage(stage::UPDATE, Self::teleport_on_click.system())
            .add_system_to_stage(stage::UPDATE, Self::spawn_on_click.system())
            .add_stage_before(
                stage::POST_UPDATE,
                "show_quads",
                SystemStage::single(Self::show_quads.system()),
            );
    }
}
