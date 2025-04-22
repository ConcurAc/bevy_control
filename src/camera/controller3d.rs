use bevy::prelude::*;

#[cfg(feature = "avian3d")]
use avian3d::prelude::*;

use crate::input::DeltaBuffer;

/// A camera controller component that provides smooth camera movement and rotation
#[derive(Component)]
#[require(DeltaBuffer)]
pub struct CameraController3d {
    /// Entity ID of the camera being controlled
    pub camera: Entity,
    /// View configuration for the camera
    pub view: CameraView3d,
    /// Sensitivity of the camera controller
    pub sensitivity: f32,
    /// Offset position from the target in world space
    pub offset: Vec3,
    /// Rate at which translation decays with smooth interpolation
    translation_decay_rate: f32,
    /// Rate at which rotation decays with smooth interpolation
    rotation_decay_rate: f32,
    /// World space axis around which yaw rotation occurs
    pub yaw_axis: Dir3,
    /// Optional limit on pitch angle, stored as cosine of half the range
    pitch_range: Option<f32>,
}

impl CameraController3d {
    /// Creates a new CameraController instance with default settings:
    /// - Sensitivity: 1.0
    /// - No offset
    /// - No smoothing (instant movement)
    /// - Yaw around Y axis
    /// - No pitch limits
    ///
    /// # Arguments
    /// * `camera` - Entity ID of the camera to control
    /// * `view` - The initial view configuration for the camera
    pub fn new(camera: Entity, view: CameraView3d) -> Self {
        Self {
            camera,
            view,

            sensitivity: 1.0,
            offset: Vec3::ZERO,

            translation_decay_rate: f32::INFINITY,
            rotation_decay_rate: f32::INFINITY,

            yaw_axis: Dir3::Y,
            pitch_range: None,
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
        self.translation_decay_rate = decay_rate;
        self.rotation_decay_rate = decay_rate;
        self
    }

    /// Sets smoothing factor for translation only.
    /// Larger values give smoother movement.
    ///
    /// # Arguments
    /// * `smoothing` - Smoothing factor for translation movement
    #[inline]
    pub fn with_translation_smoothing(mut self, smoothing: f32) -> Self {
        self.translation_decay_rate = 1.0 / smoothing;
        self
    }

    /// Sets smoothing factor for rotation only.
    /// Larger values give smoother movement.
    ///
    /// # Arguments
    /// * `smoothing` - Smoothing factor for rotational movement
    #[inline]
    pub fn with_rotation_smoothing(mut self, smoothing: f32) -> Self {
        self.rotation_decay_rate = 1.0 / smoothing;
        self
    }

    /// Sets the world space axis for yaw rotation
    ///
    /// # Arguments
    /// * `yaw_axis` - The axis around which yaw rotation occurs
    #[inline]
    pub fn with_yaw_axis(mut self, yaw_axis: Dir3) -> Self {
        self.yaw_axis = yaw_axis;
        self
    }

    /// Sets the maximum pitch angle in radians from horizontal
    ///
    /// # Arguments
    /// * `pitch_range` - Maximum pitch angle in radians (+/- from horizontal)
    #[inline]
    pub fn with_pitch_range(mut self, pitch_range: f32) -> Self {
        // stores the cosine of half the pitch range as the minimum y component
        // of the controllers yaw axis
        self.pitch_range = Some((pitch_range / 2.0).cos());
        self
    }

    /// Gets rotation delta for this frame, with smooth decay
    /// subtracting the delta from the accumulated delta
    ///
    /// # Arguments
    /// * `delta_buffer` - Delta buffer to decay
    /// * `dt` - Time elapsed since last update in seconds
    pub fn get_rotation_delta(&self, delta_buffer: &mut DeltaBuffer, dt: f32) -> Vec2 {
        if self.rotation_decay_rate.is_finite() {
            delta_buffer.decay(self.rotation_decay_rate, dt) * self.sensitivity
        } else {
            delta_buffer.take() * self.sensitivity
        }
    }

    /// Gets translation delta for this frame, with smooth decay
    /// subtracting the delta from the accumulated delta
    ///
    /// # Arguments
    /// * `delta_buffer` - Delta buffer to decay
    /// * `dt` - Time elapsed since last update in seconds
    pub fn get_translation_delta(&mut self, delta_buffer: &mut DeltaBuffer, dt: f32) -> Vec2 {
        if self.translation_decay_rate.is_finite() {
            delta_buffer.decay(self.translation_decay_rate, dt) * self.sensitivity
        } else {
            delta_buffer.take() * self.sensitivity
        }
    }

    /// Checks if a pitch rotation would exceed configured angle limits
    ///
    /// # Arguments
    /// * `pitch` - Proposed pitch rotation in radians
    /// * `rotation` - Current camera rotation
    pub fn can_rotate_pitch(&self, pitch: f32, rotation: Quat) -> bool {
        match self.pitch_range {
            Some(pitch_range) => {
                let up = rotation * Quat::from_rotation_x(pitch) * self.yaw_axis;
                up.y >= pitch_range
            }
            None => true,
        }
    }
}

/// Defines how the camera views its target
#[derive(PartialEq, Clone)]
pub enum CameraView3d {
    /// Disables influence over camera. To be handled by another system
    Manual,
    /// Translates camera orthogonally to current facing direction
    Perspective,
    /// Camera follows target from a specified distance
    Follow {
        /// Distance from target to camera
        distance: f32,
        /// Distance behind camera to check for collisions
        #[cfg(feature = "avian3d")]
        back_distance: f32,
        /// Filter for collision detection
        #[cfg(feature = "avian3d")]
        collision_filter: SpatialQueryFilter,
    },
}

/// Updates camera position and rotation each frame based on controller settings
///
/// # Arguments
/// * `camera_controllers` - Query for camera controller components and their transforms
/// * `transforms` - Query for camera transforms to modify
/// * `time` - Resource providing frame timing information
/// * `spatial_query` - Optional collision detection system (avian3d feature only)
pub(crate) fn update_camera3d(
    mut camera_controllers: Query<
        (&Transform, &CameraController3d, &mut DeltaBuffer),
        Without<Camera3d>,
    >,
    mut transforms: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    #[cfg(feature = "avian3d")] spatial_query: SpatialQuery,
) {
    for (controller_transform, controller, mut delta_buffer) in camera_controllers.iter_mut() {
        // skip if camera is manually controlled
        if controller.view == CameraView3d::Manual {
            continue;
        }

        let mut camera_transform = match transforms.get_mut(controller.camera) {
            Ok(transform) => transform,
            Err(_) => continue,
        };

        // get time delta
        let dt = time.delta_secs();

        // get camera rotation delta
        let delta = controller.get_rotation_delta(&mut delta_buffer, dt);

        // apply yaw rotation around world axis
        camera_transform.rotate_axis(controller.yaw_axis, delta.x);

        // apply pitch rotation around local x axis
        if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
            camera_transform.rotate_local_x(delta.y);
        }

        // calculate target position with offset
        let local_offset = controller_transform.rotation * controller.offset;
        let target_translation = controller_transform.translation + local_offset;

        match &controller.view {
            CameraView3d::Perspective => {
                if controller.translation_decay_rate.is_finite() {
                    // apply smoothed translation for perspective view
                    let target_distance = 0.0;

                    let mut distance = camera_transform.translation.distance(target_translation);

                    distance.smooth_nudge(&target_distance, controller.translation_decay_rate, dt);

                    camera_transform.translation = camera_transform.rotation
                        * Vec3::ZERO.with_z(distance)
                        + target_translation;
                } else {
                    // snap to target position when smoothing is disabled
                    camera_transform.translation = target_translation;
                }
            }
            CameraView3d::Follow {
                distance,
                #[cfg(feature = "avian3d")]
                back_distance,
                #[cfg(feature = "avian3d")]
                collision_filter,
            } => {
                // calculate target distance with smoothing if enabled
                let target_distance = if controller.translation_decay_rate.is_finite() {
                    let mut current = camera_transform.translation.distance(target_translation);
                    current.smooth_nudge(distance, controller.translation_decay_rate, dt);
                    current
                } else {
                    *distance
                };

                // handle collision detection if avian3d feature is enabled
                #[cfg(feature = "avian3d")]
                let target_distance = match spatial_query.cast_ray(
                    target_translation,
                    camera_transform.back(),
                    target_distance + back_distance,
                    false,
                    collision_filter,
                ) {
                    Some(hit) => target_distance.clamp(0., hit.distance) - back_distance,
                    None => target_distance,
                };

                // position camera at calculated distance behind target
                camera_transform.translation = camera_transform.rotation
                    * Vec3::ZERO.with_z(target_distance)
                    + target_translation;
            }
            _ => (),
        }
    }
}
