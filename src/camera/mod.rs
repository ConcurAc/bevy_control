mod controller;
mod input;

pub use controller::{CameraAnchor, CameraController, CameraView};

use bevy::prelude::*;

/// Camera Plugin for managing camera systems and physics plugins (when avian3d feature is enabled).
#[derive(Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_camera.before(TransformSystem::TransformPropagate),
        );
    }
}

/// Updates camera position and rotation each frame based on controller settings
///
/// # Arguments
/// * `camera_controllers` - Query for camera controller components and their transforms
/// * `transforms` - Query for camera transforms to modify
/// * `time` - Resource providing frame timing information
/// * `spatial_query` - Optional collision detection system (avian3d feature only)
pub(crate) fn update_camera(
    mut camera_controllers: Query<(&CameraController, &mut CameraBuffer)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (controller, mut camera_buffer) in camera_controllers.iter_mut() {
        // skip if camera is manually controlled
        if controller.view == CameraView::Free {
            continue;
        }

        let mut camera_transform = match transforms.get_mut(controller.camera) {
            Ok(transform) => transform,
            Err(_) => continue,
        };

        // get time delta
        let dt = time.delta_secs();

        let target_translation = match controller.view {
            CameraView::Free => camera_transform.translation,
            CameraView::Target(target) => {
                let target_transform = transforms.get(target).unwrap();
                let local_offset = target_transform.rotation * controller.offset;
                target_transform.translation + local_offset
            }
        };

        match controller.anchor {
            CameraAnchor::Free | CameraAnchor::Point => {
                // get camera rotation delta
                let delta = controller.get_rotation_delta(&mut delta_buffer, dt);

                // apply yaw rotation around world axis
                camera_transform.rotate_axis(controller.yaw_axis, delta.x);

                // apply pitch rotation around local x axis
                if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
                    camera_transform.rotate_local_x(delta.y);
                }
                let decay_rate = controller.get_translation_decay_rate();
                // calculate target distance with smoothing if enabled

                let target_distance = 0.0;

                let distance = if decay_rate.is_finite() {
                    // apply smoothed translation for perspective view
                    let mut distance = camera_transform.translation.distance(target_translation);
                    distance.smooth_nudge(&target_distance, decay_rate, dt);
                    distance
                } else {
                    target_distance
                };
                // position camera at calculated distance behind target
                camera_transform.translation = camera_transform.rotation
                    * Vec3::ZERO.with_z(target_distance)
                    + target_translation;
            }
            CameraAnchor::Plane { normal } => {}
            CameraAnchor::Orbit {
                distance: target_distance,
            } => {
                // get camera rotation delta
                let delta = controller.get_rotation_delta(&mut delta_buffer, dt);

                // apply yaw rotation around world axis
                camera_transform.rotate_axis(controller.yaw_axis, delta.x);

                // apply pitch rotation around local x axis
                if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
                    camera_transform.rotate_local_x(delta.y);
                }
                let decay_rate = controller.get_translation_decay_rate();
                // calculate target distance with smoothing if enabled

                let decay_rate = controller.get_translation_decay_rate();
                let distance = if decay_rate.is_finite() {
                    // apply smoothed translation for perspective view
                    let mut distance = camera_transform.translation.distance(target_translation);
                    distance.smooth_nudge(&target_distance, decay_rate, dt);
                    distance
                } else {
                    target_distance
                };
                // position camera at calculated distance behind target
                camera_transform.translation = camera_transform.rotation
                    * Vec3::ZERO.with_z(target_distance)
                    + target_translation;
            }
        }
    }
}
