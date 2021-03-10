use crate::physics::{bodies::AxisAlignedBoundingBox, Mass, PhysicsBundle};
use bevy::prelude::*;
use game_controller::PlayerBundle;
use tracing::instrument;

pub struct PlayerPlugin;

impl PlayerPlugin {
    #[instrument(skip(commands, materials))]
    fn add_player(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    size: Vec2::new(0.9, 0.9),
                    ..Default::default()
                },
                material: materials.add(ColorMaterial::color(Color::BLUE)),
                ..Default::default()
            })
            .with_bundle(PlayerBundle::default())
            .with_bundle(PhysicsBundle {
                bounds: AxisAlignedBoundingBox::from_center(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.9, 0.9),
                ),
                mass: Mass(62.0),
                ..Default::default()
            });
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(stage::STARTUP, Self::add_player.system());
    }
}
