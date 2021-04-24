use crate::physics::{bodies::AxisAlignedBoundingBox, Mass, PhysicsBundle};
use game_controller::{Player, PlayerBundle};
use game_core::{modes::ModeExt, GameStage, GlobalMode, ModeEvent};
use game_lib::{
    bevy::{ecs as bevy_ecs, prelude::*},
    tracing::{self, instrument},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct PlayerPlugin;

impl PlayerPlugin {
    #[instrument(skip(commands, materials))]
    fn add_player(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands.spawn_bundle(PlayerBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    size: Vec2::new(0.9, 0.9),
                    ..Default::default()
                },
                material: materials.add(ColorMaterial::color(Color::BLUE)),
                ..Default::default()
            },
            physics_bundle: PhysicsBundle {
                bounds: AxisAlignedBoundingBox::from_center(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.9, 0.9),
                ),
                mass: Mass(62.0),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    #[instrument(skip(commands, players))]
    fn remove_player(mut commands: Commands, players: Query<Entity, With<Player>>) {
        for entity in players.iter() {
            commands.entity(entity).despawn();
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            GameStage::PreUpdate,
            SystemSet::new()
                .label(PlayerPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Enter))
                .with_system(Self::add_player.system()),
        )
        .add_system_set_to_stage(
            GameStage::PostUpdate,
            SystemSet::new()
                .label(PlayerPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Exit))
                .with_system(Self::remove_player.system()),
        );
    }
}
