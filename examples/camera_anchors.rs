use bevy::{input::mouse::MouseMotion, prelude::*};

use bevy_control::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraPlugin))
        .add_systems(
            Startup,
            (setup_ui, setup_environment, setup_camera_controller),
        )
        .add_systems(Update, (update_buffer, move_controller, switch_anchor))
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
    ));

    // Create a shared cube mesh that will be reused
    let cube = Cuboid::new(1.0, 1.0, 1.0);
    let cube_mesh = meshes.add(cube);

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
        ));
    }
}

fn update_buffer(
    mut query: Query<&mut CameraBuffer>,
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

fn switch_anchor(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(Entity, &mut CameraController)>,
    transforms: Query<&Transform>,
) {
    for (entity, mut controller) in controllers.iter_mut() {
        if input.just_pressed(KeyCode::Digit1) {
            controller.anchor = CameraAnchor::Yaw;
        } else if input.just_pressed(KeyCode::Digit2) {
            let transform = transforms.get(controller.camera).unwrap();
            controller.anchor = CameraAnchor::Plane {
                normal: transform.forward(),
            };
        } else if input.just_pressed(KeyCode::Digit3) {
            controller.anchor = CameraAnchor::Point;
        } else if input.just_pressed(KeyCode::Digit4) {
            let [controller_transform, camera_transform] =
                transforms.get_many([entity, controller.camera]).unwrap();
            let distance = controller_transform
                .translation
                .distance(camera_transform.translation);
            controller.anchor = CameraAnchor::Orbit { distance };
        }
    }
}

fn move_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(&CameraController, &mut Transform), Without<Camera>>,
    camera: Query<&Transform, With<Camera>>,
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
            "Camera Anchor Controls:\n\
                1: Yaw (3D)\n\
                2: Plane (2D Panning)\n\
                3: Point (3D first person)\n\
                4: Orbit (3D third person)",
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
        CameraController::new(camera, CameraAnchor::default(), CameraView::Free)
            .with_pitch_range(f32::to_radians(90.0))
            .with_smoothing(0.05),
    ));
}
