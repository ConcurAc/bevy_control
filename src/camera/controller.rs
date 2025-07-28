use bevy::prelude::*;

/// A camera controller component that provides smooth camera movement and rotation
#[derive(Component)]
#[require(CameraBuffer)]
pub struct CameraController {
    /// Entity ID of the camera being controlled
    pub camera: Entity,
    /// Constrain camera to either plane for 2D or orbit for 3D control
    pub anchor: CameraAnchor,
    /// View configuration for the camera
    pub view: CameraView,
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

impl CameraController {
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
    pub fn new(camera: Entity, anchor: CameraAnchor, view: CameraView) -> Self {
        Self {
            camera,
            anchor,
            view,

            sensitivity: 1.0,
            offset: Vec3::ZERO,

            translation_decay_rate: f32::INFINITY,
            rotation_decay_rate: f32::INFINITY,

            yaw_axis: Dir3::Y,
            pitch_range: None,
        }
    }

    #[inline]
    pub fn get_translation_decay_rate(&self) -> f32 {
        self.translation_decay_rate
    }

    #[inline]
    pub fn get_rotation_decay_rate(&self) -> f32 {
        self.rotation_decay_rate
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
    pub fn get_rotation_delta(&self, delta_buffer: &mut CameraBuffer, dt: f32) -> Vec2 {
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
    pub fn get_translation_delta(&self, delta_buffer: &mut CameraBuffer, dt: f32) -> Vec2 {
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
            _ => true,
        }
    }
}

#[derive(Default, Clone)]
pub enum CameraAnchor {
    /// Constrains camera to plane to allow for 2D panning control
    Plane { normal: Dir3 },
    #[default]
    /// Constrains camera to point with respect to controller for first person control
    Point,
    /// Constrains camera to radial orbit around controller to allow for 3D third person control
    Orbit { distance: f32 },
}

#[derive(Default, Clone)]
pub enum CameraView {
    #[default]
    /// Allows for camera view to be dependent on input
    Free,
    /// Constrains camera to look at an Entity
    Target(Entity),
}

/// A buffer component that stores and manages a 2D vector delta value
#[derive(Component, Default)]
pub struct CameraBuffer {
    /// The current accumulated delta value
    delta: Vec2,
}

impl CameraBuffer {
    /// Adds the given delta to the buffer's current value
    #[inline]
    pub fn update(&mut self, delta: Vec2) {
        self.delta += delta;
    }

    /// Subtracts the given delta from the buffer's current value
    #[inline]
    pub fn consume(&mut self, delta: Vec2) {
        self.delta -= delta;
    }

    /// Resets the buffer's delta value to zero
    #[inline]
    pub fn reset(&mut self) {
        self.delta = Vec2::ZERO;
    }

    /// Returns the current delta value without modifying it
    #[inline]
    pub fn read(&self) -> Vec2 {
        self.delta
    }

    /// Returns the current delta value and resets the buffer
    #[inline]
    pub fn take(&mut self) -> Vec2 {
        let taken = self.delta;
        self.reset();
        taken
    }

    /// Reduces the delta value using smooth interpolation
    ///
    /// # Arguments
    /// * `rate` - The rate at which to decay the value
    /// * `dt` - The time increment
    #[inline]
    pub fn decay(&mut self, rate: f32, dt: f32) -> Vec2 {
        let mut consumed = Vec2::ZERO;
        consumed.smooth_nudge(&self.delta, rate, dt);
        self.consume(consumed);
        consumed
    }
}
