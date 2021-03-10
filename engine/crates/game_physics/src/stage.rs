/// Stage that runs before any physics steps are taken.
pub const PRE_PHYSICS_STAGE: &'static str = "pre_physics";

/// Stage that runs every physics step (0..N times per tick).
pub const PHYSICS_STAGE: &'static str = "physics";

/// Stage that runs after every physics step.
pub const POST_PHYSICS_STAGE: &'static str = "post_physics";
