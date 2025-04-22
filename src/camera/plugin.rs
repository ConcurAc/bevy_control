use bevy::prelude::*;

#[cfg(feature = "3d")]
use super::controller3d::update_camera3d;

/// Camera Plugin for managing camera systems and physics plugins (when avian3d feature is enabled).
#[derive(Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                #[cfg(feature = "3d")]
                update_camera3d.before(TransformSystem::TransformPropagate),
            ),
        );
    }
}
