use crate::ScaledOrthographicProjection;
use bevy::{
    prelude::*,
    render::camera::{CameraProjection, PerspectiveProjection},
};

pub trait ProjectionExt: CameraProjection {
    /// Gets the clip space to view space matrix. This is often the inverse
    /// of the projection matrix.
    fn get_clip_view_matrix(&self) -> Mat4 {
        self.get_projection_matrix().inverse()
    }

    /// Converts a screen coordinate to a ray in world space.
    ///
    /// # Returns
    /// A pair containing the (near, far) vectors in the resulting ray. For
    /// orthogonal projection matrices, the only difference between the two
    /// vectors will be the z components.
    fn screen_to_world(
        &self,
        camera_transform: &Transform,
        screen_position: Vec2,
        screen_size: Vec2,
    ) -> (Vec3, Vec3) {
        // Based on https://computergraphics.stackexchange.com/a/9960

        // Clip space vectors
        let pos_clip = (screen_position / screen_size) * 2.0 - Vec2::one();
        let near_clip: Vec4 = pos_clip.extend(-1.0).extend(1.0);
        let far_clip: Vec4 = pos_clip.extend(1.0).extend(1.0);

        // Inverse transformation/projection matrices
        let m_cv = self.get_clip_view_matrix(); // clip -> view
        let m_vw = camera_transform.compute_matrix(); // view -> world

        // Convert clip space coordinates to view space coordinates
        let mut near_view = m_cv * near_clip;
        near_view /= near_view[3];
        let mut far_view = m_cv * far_clip;
        far_view /= far_view[3];

        // Convert view space coordinates to world space coordinates
        let near_world = m_vw * near_view;
        let far_world = m_vw * far_view;

        (near_world.into(), far_world.into())
    }
}

impl ProjectionExt for PerspectiveProjection {}

impl ProjectionExt for ScaledOrthographicProjection {
    fn get_clip_view_matrix(&self) -> Mat4 {
        Mat4::from_cols(
            Vec4::new((self.right - self.left) / 2.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, (self.top - self.bottom) / 2.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, (self.far - self.near) / 2.0, 0.0),
            // TODO: is this row correct? does it actually matter anyway?
            Vec4::new(0.0, 0.0, (self.near - self.far) / 2.0, 1.0),
        )
    }
}
