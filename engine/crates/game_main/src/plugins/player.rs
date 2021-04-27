use crate::physics::{Mass, PhysicsBundle};
use game_controller::{Player, PlayerBundle};
use game_core::{modes::ModeExt, GameStage, GlobalMode, ModeEvent};
use game_lib::{
    bevy::{ecs as bevy_ecs, prelude::*},
    tracing::{self, instrument},
};
use game_tiles::{EntityWorldPosition, EntityWorldRect};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, SystemLabel)]
pub struct PlayerPlugin;

impl PlayerPlugin {
    #[instrument(skip(commands, materials))]
    fn add_player(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        let size = Vec2::new(1.6, 2.9);
        commands.spawn_bundle(PlayerBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    size,
                    ..Default::default()
                },
                material: materials.add(ColorMaterial::color(Color::BLUE)),
                ..Default::default()
            },
            physics_bundle: PhysicsBundle {
                bounds: EntityWorldRect::from_center(
                    EntityWorldPosition::new(0.0, 0.0),
                    (size / 2.0).into(),
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
            GameStage::GamePreUpdate,
            SystemSet::new()
                .label(PlayerPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Enter))
                .with_system(Self::add_player.system()),
        )
        .add_system_set_to_stage(
            GameStage::GamePostUpdate,
            SystemSet::new()
                .label(PlayerPlugin)
                .with_run_criteria(GlobalMode::InGame.on(ModeEvent::Exit))
                .with_system(Self::remove_player.system()),
        );
    }
}
