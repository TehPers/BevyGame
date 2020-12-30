use crate::plugins::{
    input::ActionInput,
    physics::{AxisAlignedBoundingBox, Forces, Mass, PhysicsBundle},
};
use bevy::prelude::*;
use tracing::instrument;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Player;

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
            .with(Player)
            .with_bundle(PhysicsBundle {
                bounds: AxisAlignedBoundingBox::from_center(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.9, 0.9),
                ),
                mass: Mass(62.0),
                ..Default::default()
            });
    }

    #[instrument(skip(input, query))]
    fn move_player(input: Res<Input<ActionInput>>, mut query: Query<&mut Forces, With<Player>>) {
        // Get direction to move
        const MOVE_SPEED: f32 = 5.0;
        let mut force = Vec2::default();
        if input.pressed(ActionInput::PlayerLeft) {
            force -= Vec2::unit_x() * MOVE_SPEED;
        }
        if input.pressed(ActionInput::PlayerRight) {
            force += Vec2::unit_x() * MOVE_SPEED;
        }
        if input.just_pressed(ActionInput::PlayerJump) {
            force += Vec2::unit_y() * 10.0;
        }

        // Apply force
        if force.length_squared() >= 0.1 {
            for mut forces in query.iter_mut() {
                forces.0.push(force);
            }
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(stage::STARTUP, Self::add_player.system())
            .add_system_to_stage(stage::UPDATE, Self::move_player.system());
    }
}
