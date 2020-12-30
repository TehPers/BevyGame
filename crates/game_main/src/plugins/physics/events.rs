use bevy::prelude::*;

/// A collision between two bodies.
#[derive(Clone, Debug)]
pub struct CollisionEvent {
    /// Entities involved with the collision.
    pub entities: [Entity; 2],

    /// Masses of each of the entities involved in `kg`.
    pub masses: [f32; 2],

    /// Initial velocities of the entities involved in `m/s`.
    pub initial_velocities: [Vec2; 2],
}

impl CollisionEvent {
    /// Calculates the final momentums of the entities involved in a collision
    /// in `kg * m/s`.
    ///
    /// # Arguments
    /// * `restitution` - The coefficient of restitution in the range `[0, 1]`.
    ///                   A restitution of 0 = inelastic and 1 = elastic.
    pub fn calculate_collision(&self, restitution: f32) -> [Vec2; 2] {
        let [m1, m2] = self.masses;
        let [v1, v2] = self.initial_velocities;
        let (p1, p2) = (m1 * v1, m2 * v2);

        let mass_sum = m1 + m2;
        let velocity_diff = v1 - v2;
        [
            (restitution * m2 * velocity_diff + p1 + p2) / mass_sum,
            (restitution * m1 * -velocity_diff + p1 + p2) / mass_sum,
        ]
    }
}
