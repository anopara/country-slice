// TODO: Part 1. SETUP
// 1. DONE setup Bevy renderer
// 2. DONE import a mesh
// 3. DONE render the mesh
// 4. DONE setup the camera with dolly
// 5. DONE draw a SOMETHING with the mouse

//----
// 1. for funs, fix normals (make a gif to record progress)
// 2. make a prototype in Houdini of the wall setup - make an implementation plan

mod curve;
mod curve_manager;
mod utils;
mod wall_constructor;

use bevy::{
    prelude::*,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
    },
};
use bevy_mod_picking::{PickableBundle, PickingCamera, PickingCameraBundle, PickingPlugin};

use bevy_dolly::Transform2Bevy;

use curve::Curve;
use curve_manager::CurveManager;
use dolly::prelude::{Arm, CameraRig, Smooth, YawPitch};
use wall_constructor::WallConstructor;

// Give camera a component so we can find it and update with Dolly rig
struct MainCamera;

// Mark the cube that is the preview of mouse raycast intersection
struct PreviewCube;

struct CustomMesh;

// mark the bricks, so that can be deleted and recreated
struct BrickEntity;

fn main() {
    /*
    let mut points: Vec<_> = (0..=10).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();
    points.push(Vec3::new(0.0, 0.0, 0.0));
    let c = Curve::from(points);

    println!("{:?}", c.get_tangent_at_u(0.5));
    */

    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .insert_resource(CurveManager::new())
        .add_startup_system(setup.system())
        .add_system(update_camera.system())
        .add_system(handle_mouse_clicks.system())
        .add_system(query_intersection.system().label("intersection"))
        .add_system(update_wall.system().after("intersection"))
        .run();
}

fn update_wall(
    mut commands: Commands,
    curve_manager: ResMut<CurveManager>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bricks_query: Query<Entity, With<BrickEntity>>,
) {
    // delete old breaks if there are any
    for entity in bricks_query.iter() {
        commands.entity(entity).despawn()
    }

    if curve_manager.point_positions.len() > 1 {
        // take the curve
        let curve = Curve::from(curve_manager.smooth_positions());
        let bricks = WallConstructor::from_curve(&curve);

    
        for brick in &bricks {
            let transform = Transform {
                translation: brick.position,
                rotation: brick.rotation,
                scale: brick.scale,
            };

            commands
                .spawn_bundle(PbrBundle {
                    mesh: curve_manager.brick_mesh_handle.clone().unwrap(),
                    material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                    transform,
                    ..Default::default()
                })
                .insert(BrickEntity);
        }
    
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut curve_manager: ResMut<CurveManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    // load brick mesh
    curve_manager.brick_mesh_handle = Some(meshes.add(
        utils::load_gltf_as_bevy_mesh_w_vertex_color("assets/brick.glb"),
    ));

    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // floor
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(utils::load_gltf_as_bevy_mesh_w_vertex_color(
                "assets/floor.glb",
            )),
            material: asset_server.load("test.glb#Material0"),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());

    // preview cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                base_color_texture: None,
                roughness: 1.0,
                metallic: 0.0,
                metallic_roughness_texture: None,
                reflectance: 0.0,
                normal_map: None,
                double_sided: true,
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
            .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-35.0))
            .with(Smooth::new_rotation(1.5))
            .with(Arm::new(dolly::glam::Vec3::Z * 9.0))
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut curve_manager: ResMut<CurveManager>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<&mut PickingCamera>,
    mut cube_query: Query<(
        &mut PreviewCube,
        &mut bevy::transform::components::Transform,
    )>,
) {
    // Update Preview Cube
    for camera in query.iter_mut() {
        if let Some((_, intersection)) = camera.intersect_top() {
            for (_, mut transform) in cube_query.iter_mut() {
                transform.translation = intersection.position();

                if let Some(mesh_handle) = curve_manager.preview_mesh_handle.as_ref() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        curve_manager.populate_bevy_mesh(mesh);

                        if let Some(last_point) = curve_manager.point_positions.last_mut() {
                            *last_point = intersection.position();
                        }
                    }
                }
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        curve_manager.preview_mesh_handle = None;
        curve_manager.point_positions = Vec::new();
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
        for camera in query.iter_mut() {
            if let Some((_, intersection)) = camera.intersect_top() {
                curve_manager.point_positions.push(intersection.position());

                // If we just made exactly 2 points, create a mesh
                if curve_manager.point_positions.len() == 2 {
                    let mut mesh =
                        Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

                    curve_manager.populate_bevy_mesh(&mut mesh);
                    let handle = meshes.add(mesh);
                    curve_manager.preview_mesh_handle = Some(handle.clone());

                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: handle,
                            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                            ..Default::default()
                        })
                        .insert(CustomMesh);
                }
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

const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_color = Vertex_Color;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
void main() {
    o_Target = vec4(v_color, 1.0);
}
"#;
