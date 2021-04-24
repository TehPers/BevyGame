use crate::bodies::AxisAlignedBoundingBox;
use game_lib::{
    bevy::{ecs as bevy_ecs, prelude::*},
    derive_more::{Display, From, Into},
};
use game_tiles::EntityWorldPosition;

/// All the components needed for an entity to be registered with the physics
/// engine.
#[derive(Bundle, Default)]
pub struct PhysicsBundle {
    pub bounds: AxisAlignedBoundingBox,
    pub body_type: BodyType,
    pub mass: Mass,

    pub forces: Forces,
    pub acceleration: Acceleration,
    pub velocity: Velocity,
}

/// How the body should be treated by the physics engine.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Reflect)]
#[reflect(Component)]
pub enum BodyType {
    /// Body is affected by forces, acceleration, etc. and collisions should be
    ///  calculated for this body.
    Kinematic,

    /// Body cannot move, however other kinematic bodies can still collide with
    /// it.
    Static,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Kinematic
    }
}

/// Velocity in `m/s`. This is **not** reset once it has been applied by the
/// physics engine.
#[derive(Clone, Copy, PartialEq, Debug, Default, From, Into, Reflect)]
#[reflect(Component)]
pub struct Velocity(pub EntityWorldPosition);

/// Acceleration in `m/s^2`. This is reset to the null vector once it has been
/// applied by the physics engine.
#[derive(Clone, Copy, PartialEq, Debug, Default, From, Into, Reflect)]
#[reflect(Component)]
pub struct Acceleration(pub EntityWorldPosition);

/// Force in `kg * m/s^2`. This is cleared once it has been applied by the
/// physics engine.
#[derive(Clone, PartialEq, Debug, Default, From, Into, Reflect)]
#[reflect(Component)]
pub struct Forces(pub Vec<EntityWorldPosition>);

/// Mass in `kg`. A non-positive mass is invalid and may cause the
/// simulation to panic or behave strangely.
#[derive(Clone, Copy, PartialEq, Debug, Display, From, Into, Reflect)]
#[reflect(Component)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Mass(62.0)
    }
}

/// Gravitational acceleration in `m/s^2`. This overrides the global
/// gravitational acceleration, allowing individual bodies to have their own
/// gravity.
#[derive(Clone, Copy, PartialEq, Debug, From, Into, Reflect)]
#[reflect(Component)]
pub struct Gravity(pub EntityWorldPosition);

impl Default for Gravity {
    fn default() -> Self {
        Gravity(EntityWorldPosition::Y * -9.81)
    }
}

/// Drag coefficient. Since neither the shape of the body nor the density of
/// the fluid it is in are taken into account, this coefficient has the unit
/// `kg/m` (equivalent to `m^2 * kg/m^3`).
#[derive(Clone, Copy, PartialEq, Debug, Display, From, Into, Reflect)]
#[reflect(Component)]
pub struct Drag(pub f32);

impl Default for Drag {
    fn default() -> Self {
        Self::from_terminal_velocity(55.56, 62.0, 9.81)
    }
}

impl Drag {
    /// Calculates simplified drag coefficient from the expected terminal
    ///  velocity of a body with the given mass and gravity. Terminal velocity
    /// in the range (-1.0, 1.0) will cause the drag coefficient to be too
    /// large.
    pub fn from_terminal_velocity(terminal_velocity: f32, mass: f32, gravity: f32) -> Self {
        // v_t = sqrt((m * g) / b), where b = p * A * C_d (simplified with this engine)
        // b = (m * g) / v_t^2
        Drag((mass * gravity) / terminal_velocity.powi(2))
    }

    pub fn get_terminal_velocity(self, mass: f32, gravity: f32) -> f32 {
        ((mass * gravity) / self.0).sqrt()
    }
}
