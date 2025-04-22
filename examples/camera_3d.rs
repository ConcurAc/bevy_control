use bevy::{input::mouse::MouseMotion, prelude::*};

use bevy_control::prelude::*;

#[cfg(feature = "avian3d")]
use avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CameraPlugin,
            #[cfg(feature = "avian3d")]
            PhysicsPlugins::default(),
        ))
        .add_systems(
            Startup,
            (setup_ui, setup_environment, setup_camera_controller),
        )
        .add_systems(Update, (update_buffer, move_controller))
        .add_systems(
            PostUpdate,
            switch_view.before(TransformSystem::TransformPropagate),
        )
        .run();
}

fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a point light above the scene
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 5.0, 0.0)));

    // Create a large white ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(100.0, 100.0)))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        #[cfg(feature = "avian3d")]
        (Collider::half_space(Vec3::Y), RigidBody::Static),
    ));

    // Create a shared cube mesh that will be reused
    let cube = Cuboid::new(1.0, 1.0, 1.0);
    let cube_mesh = meshes.add(cube);

    #[cfg(feature = "avian3d")]
    let cube_collider = Collider::from(cube);

    // Spawn cubes in a circle
    let total = 8;
    let distance = 5.0;
    for i in 0..total {
        // Calculate angle for circular placement
        let n = i as f32 / total as f32;
        let angle = 2.0 * std::f32::consts::PI * n;
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(materials.add(Color::BLACK.lighter(n))),
            // Position cube using trigonometry for circular arrangement
            Transform::from_xyz(angle.cos() * distance, 0.5, angle.sin() * distance),
            #[cfg(feature = "avian3d")]
            (cube_collider.clone(), RigidBody::Static),
        ));
    }
}

fn update_buffer(
    mut query: Query<&mut DeltaBuffer>,
    mut mouse: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for mut delta_buffer in query.iter_mut() {
        // Calculate total mouse movement this frame
        let delta = -mouse.read().map(|event| event.delta).sum::<Vec2>();

        // Update input buffer based on mouse movement
        delta_buffer.update(delta * time.delta_secs());
    }
}

fn switch_view(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(Entity, &mut CameraController3d, &mut DeltaBuffer)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (_entity, mut controller, mut delta_buffer) in controllers.iter_mut() {
        if input.just_pressed(KeyCode::Digit1) {
            controller.view = CameraView3d::Perspective;
        } else if input.just_pressed(KeyCode::Digit2) {
            controller.view = CameraView3d::Follow {
                distance: 10.0,
                #[cfg(feature = "avian3d")]
                back_distance: 0.2,
                #[cfg(feature = "avian3d")]
                collision_filter: SpatialQueryFilter::from_excluded_entities([_entity]),
            };
        } else if input.pressed(KeyCode::Digit8) {
            // only works if view is manual
            controller.view = CameraView3d::Manual;
            // custom panning
            let mut camera_transform = transforms.get_mut(controller.camera).unwrap();

            let delta = controller.get_translation_delta(&mut delta_buffer, time.delta_secs());
            let translation = camera_transform.rotation * Vec3::new(delta.x, -delta.y, 0.0);

            camera_transform.translation += translation;
        } else if input.pressed(KeyCode::Digit9) {
            // only works if view is manual
            controller.view = CameraView3d::Manual;
            // custom pivot
            let mut camera_transform = transforms.get_mut(controller.camera).unwrap();

            let delta = controller.get_rotation_delta(&mut delta_buffer, time.delta_secs());

            // apply yaw rotation
            camera_transform.rotate_axis(controller.yaw_axis, delta.x);

            // apply pitch rotation (around local x axis)
            if controller.can_rotate_pitch(delta.y, camera_transform.rotation) {
                camera_transform.rotate_local_x(delta.y);
            }
        } else if input.just_pressed(KeyCode::Digit0) {
            // set to manual to do nothing
            controller.view = CameraView3d::Manual;
        } else if controller.view == CameraView3d::Manual {
            // manually consume all the unused input in the buffer
            delta_buffer.reset();
        }
    }
}

fn move_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(&CameraController3d, &mut Transform), Without<Camera3d>>,
    camera: Query<&Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    for (controller, mut transform) in controllers.iter_mut() {
        let camera_transform = camera.get(controller.camera).unwrap();

        let mut direction = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            direction += Vec2::NEG_Y;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            direction += Vec2::Y;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            direction += Vec2::NEG_X;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            direction += Vec2::X;
        }
        // skip to avoid normalizing zero vector
        if direction != Vec2::ZERO {
            let mut delta = camera_transform.rotation * Vec3::new(direction.x, 0.0, direction.y);
            // make input along the plane with yaw axis normal
            delta = delta
                .reject_from_normalized(controller.yaw_axis.as_vec3())
                .normalize();

            transform.translation += delta * time.delta_secs();
        }
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Node::DEFAULT).with_children(|parent| {
        parent.spawn(Text::new(
            "Camera Controls:\n\
                1: Perspective View\n\
                2: Follow View\n\
                8 (hold): Manual Pan Camera\n\
                9 (hold): Manual Rotate Camera\n\
                0: Manual View Mode",
        ));
    });
}

fn setup_camera_controller(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn camera and get entity
    let camera = commands.spawn(Camera3d::default()).id();

    let cube = Cuboid::new(0.5, 0.5, 0.5);
    let cube_mesh = meshes.add(cube);

    commands.spawn((
        Transform::from_xyz(0.0, 0.5, 0.0),
        Mesh3d(cube_mesh),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.3, 10.0, 0.3))),
        // add camera controller component
        CameraController3d::new(camera, CameraView3d::Perspective)
            .with_pitch_range(f32::to_radians(90.0))
            .with_sensitivity(0.25)
            .with_smoothing(0.05),
    ));
}
