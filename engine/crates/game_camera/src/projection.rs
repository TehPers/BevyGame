use bevy::{
    prelude::*,
    reflect::Reflect,
    render::camera::{CameraProjection, DepthCalculation, WindowOrigin},
};

#[derive(Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScaledOrthographicProjection {
    /// Automatically calculated from zoom.
    pub left: f32,

    /// Automatically calculated from zoom.
    pub right: f32,

    /// Automatically calculated from zoom.
    pub top: f32,

    /// Automatically calculated from zoom.
    pub bottom: f32,

    pub near: f32,
    pub far: f32,

    /// The origin of the camera with respect to window size.
    pub window_origin: WindowOrigin,

    /// The factor each object's width and height are multiplied by.
    /// This value cannot be zero, otherwise rendering will panic.
    pub zoom: f32,
}

impl CameraProjection for ScaledOrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        match self.window_origin {
            WindowOrigin::Center => {
                let half_width = width / (2.0 * self.zoom);
                let half_height = height / (2.0 * self.zoom);
                self.left = -half_width;
                self.right = half_width;
                self.top = half_height;
                self.bottom = -half_height;
            }
            WindowOrigin::BottomLeft => {
                self.left = 0.0;
                self.right = width / self.zoom;
                self.top = height / self.zoom;
                self.bottom = 0.0;
            }
        }
    }

    fn depth_calculation(&self) -> DepthCalculation {
        DepthCalculation::ZDifference
    }
}

impl Default for ScaledOrthographicProjection {
    fn default() -> Self {
        ScaledOrthographicProjection {
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
            top: 0.0,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::Center,
            zoom: 1.0,
        }
    }
}
