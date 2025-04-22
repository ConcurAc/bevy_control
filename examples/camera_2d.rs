use bevy::{input::mouse::MouseMotion, prelude::*};

use bevy_control::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraPlugin))
        .add_systems(
            Startup,
            (setup_environment, setup_camera_controller, setup_ui),
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
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Add a point light above the scene
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 5.0, 0.0)));

    // Create a shared rectangle mesh that will be reused
    let rectangle = Rectangle::new(1.0, 1.0);
    let rectangle_mesh = meshes.add(rectangle);

    // Spawn cubes in a circle
    let total = 8;
    let distance = 5.0;
    for i in 0..total {
        // Calculate angle for circular placement
        let n = i as f32 / total as f32;
        let angle = 2.0 * std::f32::consts::PI * n;
        commands.spawn((
            Mesh2d(rectangle_mesh.clone()),
            MeshMaterial2d(materials.add(Color::BLACK.lighter(n))),
            // Position cube using trigonometry for circular arrangement
            Transform::from_xyz(angle.cos() * distance, angle.sin() * distance, i as f32),
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

const ZOOM_ADJUSTMENT_FACTOR: f32 = 0.01;

fn switch_view(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(Entity, &mut CameraController2d, &mut DeltaBuffer)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (_entity, mut controller, mut delta_buffer) in controllers.iter_mut() {
        if input.just_pressed(KeyCode::Digit1) {
            controller.view = CameraView2d::Follow { distance: 1.0 };
        } else if input.pressed(KeyCode::KeyQ) {
            // zoom in
            controller.zoom_by(1.0 + ZOOM_ADJUSTMENT_FACTOR);
        } else if input.pressed(KeyCode::KeyE) {
            // zoom out
            controller.zoom_by(1.0 - ZOOM_ADJUSTMENT_FACTOR);
        } else if input.pressed(KeyCode::Digit8) {
            // only works if view is manual
            controller.view = CameraView2d::Manual;
            // custom panning

            let mut camera_transform = transforms.get_mut(controller.camera).unwrap();

            let delta = controller.get_delta(&mut delta_buffer, time.delta_secs());
            let translation = camera_transform.rotation * Vec3::new(delta.x, -delta.y, 0.0);

            camera_transform.translation += translation;
        } else if input.pressed(KeyCode::Digit9) {
            // only works if view is manual
            controller.view = CameraView2d::Manual;
            // custom rotation

            let mut camera_transform = transforms.get_mut(controller.camera).unwrap();

            let delta = controller.get_delta(&mut delta_buffer, time.delta_secs());

            camera_transform.rotate_local_z(delta.y);
        } else if input.just_pressed(KeyCode::Digit0) {
            // set to manual to do nothing
            controller.view = CameraView2d::Manual;
        } else if controller.view == CameraView2d::Manual {
            // manually consume all the unused input in the buffer
            delta_buffer.reset();
        }
    }
}

fn move_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut controllers: Query<(&CameraController2d, &mut Transform), Without<Camera2d>>,
    camera: Query<&Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    for (controller, mut transform) in controllers.iter_mut() {
        let camera_transform = camera.get(controller.camera).unwrap();

        let mut direction = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            direction += Vec2::Y;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            direction += Vec2::NEG_Y;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            direction += Vec2::NEG_X;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            direction += Vec2::X;
        }

        let delta = camera_transform.rotation * Vec3::new(direction.x, direction.y, 0.0);

        transform.translation += delta * time.delta_secs();
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Node::DEFAULT).with_children(|parent| {
        parent.spawn(Text::new(
            "Camera Controls:\n\
                1: Follow View\n\
                8 (hold): Manual Pan\n\
                9 (hold): Manual Rotate\n\
                0: Manual View Mode\n\
                Q (hold): Zoom in\n\
                E (hold): Zoom out\n\
                W/A/S/D or Arrow Keys: Move controller",
        ));
    });
}

fn setup_camera_controller(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // spawn camera and get entity
    let camera = commands.spawn(Camera2d::default()).id();

    let rectangle = Rectangle::new(1.0, 1.0);
    let rectangle_mesh = meshes.add(rectangle);

    let mut entity_commands = commands.spawn((
        Mesh2d(rectangle_mesh),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.3, 10.0, 0.3))),
    ));
    entity_commands.insert(
        // add camera controller component
        CameraController2d::new(camera, CameraView2d::Follow { distance: 1.0 })
            .with_zoom(10.0)
            .with_smoothing(0.05),
    );
}
