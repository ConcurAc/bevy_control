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
    mut camera_transforms: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) -> Result<(), BevyError> {
    for (controller, mut buffer) in camera_controllers.iter_mut() {
        let mut camera_transform = camera_transforms.get_mut(controller.camera)?;
        // get time delta
        let dt = time.delta_secs();

        match controller.anchor {
            CameraAnchor::Yaw => {
                let delta = controller.get_translation_delta(&mut buffer, dt);

                let displacement =
                    controller.yaw_axis * delta.y + camera_transform.local_x() * delta.x;

                camera_transform.translation += displacement;
            }
            CameraAnchor::Plane { normal } => {
                let delta = controller.get_translation_delta(&mut buffer, dt);
                let local_y = controller
                    .yaw_axis
                    .reject_from_normalized(normal.as_vec3())
                    .normalize();
                let local_x = controller.yaw_axis.cross(normal.as_vec3()).normalize();
                let displacement = local_y * delta.y + local_x * delta.x;

                camera_transform.translation += displacement;
            }
            _ => {
                // get camera rotation delta
                let delta = controller.get_rotation_delta(&mut buffer, dt);

                // apply yaw rotation around world axis
                let yaw_rotation = Quat::from_axis_angle(controller.yaw_axis.as_vec3(), delta.x);
                buffer.rotation = yaw_rotation * buffer.rotation;

                // apply pitch rotation around local x axis
                if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
                    buffer.rotation *= Quat::from_rotation_x(delta.y);
                }
            }
        }
    }
    Ok(())
}

/// Updates camera position and rotation each frame based on controller settings
///
/// # Arguments
/// * `camera_controllers` - Query for camera controller and buffer
/// * `camera_transforms` - Query for camera transforms to modify
/// * `target_transforms` - Query for target transforms for camera targetting
/// * `time` - Resource providing frame timing information
fn update_camera(
    camera_controllers: Query<(Entity, &CameraController, &CameraBuffer)>,
    mut camera_transforms: Query<&mut Transform, With<Camera>>,
    target_transforms: Query<&Transform, Without<Camera>>,
    time: Res<Time>,
) -> Result<(), BevyError> {
    for (entity, controller, buffer) in camera_controllers.iter() {
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
                    buffer.rotation * Vec3::ZERO.with_z(distance) + target_translation;
            }
            _ => (),
        }
        match controller.view {
            CameraView::Free => {
                camera_transform.rotation = buffer.rotation;
            }
            CameraView::Target(target) => {
                let target_transform = target_transforms.get(target)?;
                camera_transform.look_at(target_transform.translation, controller.yaw_axis);
            }
        }
    }
    Ok(())
}
