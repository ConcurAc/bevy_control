mod controller;

pub use controller::{CameraAnchor, CameraBuffer, CameraController, CameraView};

use bevy::prelude::*;

/// Camera Plugin for managing camera systems and physics plugins (when avian3d feature is enabled).
#[derive(Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                consume_buffers.before(update_camera),
                update_camera.before(TransformSystem::TransformPropagate),
            ),
        );
    }
}

fn consume_buffers(
    mut camera_controllers: Query<(&CameraController, &mut CameraBuffer)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (controller, mut camera_buffer) in camera_controllers.iter_mut() {
        let mut camera_transform = match transforms.get_mut(controller.camera) {
            Ok(transform) => transform,
            Err(_) => continue,
        };

        // get time delta
        let dt = time.delta_secs();

        match controller.anchor {
            CameraAnchor::Plane { normal } => {
                let delta = controller.get_translation_delta(&mut camera_buffer, dt);

                let displacement = controller.yaw_axis * delta.y
                    + controller.yaw_axis.cross(normal.as_vec3()).normalize() * delta.x;

                camera_transform.translation += displacement;
            }
            _ => {
                // get camera rotation delta
                let delta = controller.get_rotation_delta(&mut camera_buffer, dt);

                // apply yaw rotation around world axis
                camera_transform.rotate_axis(controller.yaw_axis, delta.x);

                // apply pitch rotation around local x axis
                if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
                    camera_transform.rotate_local_x(delta.y);
                }
            }
        }
    }
}

/// Updates camera position and rotation each frame based on controller settings
///
/// # Arguments
/// * `camera_controllers` - Query for camera controller components and their transforms
/// * `transforms` - Query for camera transforms to modify
/// * `time` - Resource providing frame timing information
/// * `spatial_query` - Optional collision detection system (avian3d feature only)
fn update_camera(
    camera_controllers: Query<(Entity, &CameraController)>,
    mut camera_transforms: Query<&mut Transform, With<Camera>>,
    target_transforms: Query<&Transform, Without<Camera>>,
    time: Res<Time>,
) -> Result<(), BevyError> {
    for (entity, controller) in camera_controllers.iter() {
        let mut camera_transform = camera_transforms.get_mut(controller.camera)?;
        let controller_transform = target_transforms.get(entity)?;

        // get time delta
        let dt = time.delta_secs();

        match controller.anchor {
            CameraAnchor::Point => {
                let local_offset = controller_transform.rotation * controller.offset;
                let target_translation = controller_transform.translation + local_offset;

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
                camera_transform.translation =
                    camera_transform.rotation * Vec3::ZERO.with_z(distance) + target_translation;
            }
            CameraAnchor::Orbit {
                distance: target_distance,
            } => {
                let local_offset = controller_transform.rotation * controller.offset;
                let target_translation = controller_transform.translation + local_offset;

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
                camera_transform.translation =
                    camera_transform.rotation * Vec3::ZERO.with_z(distance) + target_translation;
            }
            _ => (),
        }
        match controller.view {
            CameraView::Free => (),
            CameraView::Target(target) => match controller.anchor {
                CameraAnchor::Plane { normal: _ } => (),
                _ => {
                    let target_transform = target_transforms.get(target)?;
                    camera_transform.look_at(target_transform.translation, controller.yaw_axis);
                }
            },
        }
    }
    Ok(())
}
