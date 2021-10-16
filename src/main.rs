// TODO: Part 1. SETUP
// 1. DONE setup Bevy renderer
// 2. import a mesh
// 3. DONE render the mesh
// 4. DONE setup the camera with dolly
// 5. draw a SOMETHING with the mouse

use bevy::prelude::*;
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DebugEventsPickingPlugin, PickableBundle, PickingCamera,
    PickingCameraBundle, PickingPlugin,
};

use bevy_dolly::Transform2Bevy;

use dolly::prelude::{Arm, CameraRig, Smooth, YawPitch};

// Give camera a component so we can find it and update with Dolly rig
struct MainCamera;

struct PreviewCube;

// Mark our generic `RayCastMesh`s and `RayCastSource`s as part of the same group, or "RayCastSet".
struct MyRaycastSet;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_startup_system(setup.system())
        .add_system(update_camera.system())
        .add_system(handle_mouse_clicks.system())
        .add_system(query_intersection.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());
    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                base_color_texture: None,
                roughness: 1.0,
                metallic: 0.0,
                metallic_roughness_texture: None,
                reflectance: 0.0,
                normal_map: None,
                double_sided: false,
                occlusion_texture: None,
                emissive: Color::rgb(1.0, 1.0, 1.0),
                emissive_texture: None,
                unlit: false,
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(PreviewCube);
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    // TODO: we can replace this with a resource and update camera ourselves?
    commands.spawn().insert(
        CameraRig::builder()
            .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-30.0))
            .with(Smooth::new_rotation(1.5))
            .with(Arm::new(dolly::glam::Vec3::Z * 5.0))
            .build(),
    );
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 10.0, 5.0)
                .looking_at(bevy::math::Vec3::ZERO, bevy::math::Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera)
        .insert_bundle(PickingCameraBundle::default());
}

fn handle_mouse_clicks(mouse_input: Res<Input<MouseButton>>, windows: Res<Windows>) {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        println!("click at {:?}", win.cursor_position());
    }
}

fn query_intersection(
    mut query: Query<&mut PickingCamera>,
    mut cube_query: Query<(
        &mut PreviewCube,
        &mut bevy::transform::components::Transform,
    )>,
) {
    for camera in query.iter_mut() {
        println!("{:?}", camera.intersect_top());

        if let Some((_, intersection)) = camera.intersect_top() {
            for (cube, mut transform) in cube_query.iter_mut() {
                transform.translation = intersection.position();
            }
        }
    }
}

fn update_camera(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: QuerySet<(
        Query<(&mut Transform, With<MainCamera>)>,
        Query<&mut CameraRig>,
    )>,
) {
    let mut rig = query.q1_mut().single_mut().unwrap();
    let camera_driver = rig.driver_mut::<YawPitch>();

    if keys.just_pressed(KeyCode::Z) {
        camera_driver.rotate_yaw_pitch(-90.0, 0.0);
    }
    if keys.just_pressed(KeyCode::X) {
        camera_driver.rotate_yaw_pitch(90.0, 0.0);
    }

    let transform = rig.update(time.delta_seconds());
    let (mut cam, _) = query.q0_mut().single_mut().unwrap();

    cam.transform_2_bevy(transform);
}
