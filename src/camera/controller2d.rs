use bevy::prelude::*;

use crate::input::DeltaBuffer;

/// A camera controller component that provides smooth camera movement and rotation
#[derive(Component)]
#[require(DeltaBuffer)]
pub struct CameraController2d {
    /// Entity ID of the camera being controlled
    pub camera: Entity,
    /// View configuration for the camera
    pub view: CameraView2d,
    /// Sensitivity of the camera controller
    pub sensitivity: f32,
    /// Offset position from the target
    pub offset: Vec3,
    /// Rate at which translation decays with smooth interpolation
    decay_rate: f32,
    /// Scale multiplier for camera projection
    scale: f32,
}

impl CameraController2d {
    /// Creates a new CameraController instance
    ///
    /// # Arguments
    /// * `camera` - Entity ID of the camera to control
    /// * `view` - View configuration for the camera
    pub fn new(camera: Entity, view: CameraView2d) -> Self {
        Self {
            camera,
            view,

            sensitivity: 1.0,
            offset: Vec3::ZERO,

            decay_rate: f32::INFINITY,
            scale: 1.0,
        }
    }

    /// Sets the sensitivity multiplier for all movement
    ///
    /// # Arguments
    /// * `sensitivity` - Multiplier for camera movement sensitivity
    #[inline]
    pub fn with_sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    /// Sets the world space offset from the target position
    ///
    /// # Arguments
    /// * `offset` - 3D vector offset from target position
    #[inline]
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Sets smoothing factor for both translation and rotation.
    /// Larger values give smoother movement.
    ///
    /// # Arguments
    /// * `smoothing` - Smoothing factor for camera movement
    #[inline]
    pub fn with_smoothing(mut self, smoothing: f32) -> Self {
        let decay_rate = 1.0 / smoothing;
        self.decay_rate = decay_rate;
        self
    }

    /// Sets scale multiplier for camera projection
    ///
    /// # Arguments
    /// * `zoom` - Zoom level for camera projection
    #[inline]
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.scale = 1.0 / zoom;
        self
    }

    /// Sets scale multiplier for camera projection
    ///
    /// # Arguments
    /// * `zoom` - Zoom level for camera projection
    #[inline]
    pub fn set_zoom(&mut self, zoom: f32) {
        self.scale = 1.0 / zoom;
    }

    /// Set zoom level for camera projection
    ///
    /// # Arguments
    /// * `zoom` - Zoom level for camera projection
    #[inline]
    pub fn zoom_by(&mut self, factor: f32) {
        self.scale /= factor;
    }

    /// Gets delta for this frame, with smooth decay
    /// subtracting the delta from the accumulated delta
    ///
    /// # Arguments
    /// * `dt` - Time elapsed since last update in seconds
    pub fn get_delta(&mut self, delta_buffer: &mut DeltaBuffer, dt: f32) -> Vec2 {
        if self.decay_rate.is_finite() {
            delta_buffer.decay(self.decay_rate, dt) * self.sensitivity
        } else {
            delta_buffer.take() * self.sensitivity
        }
    }
}

/// Defines how the camera views its target
#[derive(PartialEq)]
pub enum CameraView2d {
    /// Disables influence over camera. To be handled by another system
    Manual,
    /// Camera follows target from a specified distance
    Follow { distance: f32 },
}

/// Updates camera position and rotation each frame
///
/// # Arguments
/// * `controller_query` - Query for camera controller components
/// * `camera_query` - Query for transform components
/// * `time` - Time resource for frame timing
/// * `spatial_query` - Spatial query for collision detection (only used with avian3d feature)
pub(crate) fn update_camera2d(
    mut controller_query: Query<(&Transform, &mut CameraController2d), Without<Camera2d>>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    time: Res<Time>,
) {
    for (controller_transform, controller) in controller_query.iter_mut() {
        // skip if camera is manually controlled
        if controller.view == CameraView2d::Manual {
            continue;
        }
        let (mut camera_transform, mut projection) = match camera_query.get_mut(controller.camera) {
            Ok(components) => components,
            Err(_) => continue,
        };

        // get time delta
        let dt = time.delta_secs();

        // calculate target position with offset
        let target_translation = controller_transform.translation + controller.offset;

        match &controller.view {
            CameraView2d::Follow { distance } => {
                let difference = target_translation - camera_transform.translation;

                // Handle zoom

                if controller.decay_rate.is_finite() {
                    projection
                        .scale
                        .smooth_nudge(&controller.scale, controller.decay_rate, dt);
                } else {
                    projection.scale = controller.scale;
                }

                let current_displacement = difference.xy();
                let current_distance = current_displacement.length();

                if current_distance == 0.0 {
                    continue;
                }

                if current_distance > *distance {
                    let displacement = if controller.decay_rate.is_finite() {
                        // apply smoothed translation
                        let mut new_distance = current_distance;
                        new_distance.smooth_nudge(distance, controller.decay_rate, dt);
                        current_displacement * (current_distance - new_distance) / current_distance
                    } else {
                        current_displacement * (current_distance - *distance) / current_distance
                    };
                    camera_transform.translation.x += displacement.x;
                    camera_transform.translation.y += displacement.y;
                }
            }
            _ => (),
        }
    }
}
