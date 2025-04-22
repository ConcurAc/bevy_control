use bevy::prelude::*;

/// A buffer component that stores and manages a 2D vector delta value
#[derive(Component, Default)]
pub struct DeltaBuffer {
    /// The current accumulated delta value
    delta: Vec2,
}

impl DeltaBuffer {
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
