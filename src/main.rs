// TODO: Part 1. SETUP
// 1. DONE setup Bevy renderer
// 2. DONE import a mesh
// 3. DONE render the mesh
// 4. DONE setup the camera with dolly
// 5. draw a SOMETHING with the mouse

use bevy::{
    prelude::*,
    render::{
        mesh::{shape, VertexAttributeValues},
        pipeline::{PipelineDescriptor, RenderPipeline},
        shader::{ShaderStage, ShaderStages},
    },
};
use bevy_mod_picking::{PickableBundle, PickingCamera, PickingCameraBundle, PickingPlugin};

use bevy_dolly::Transform2Bevy;

use dolly::prelude::{Arm, CameraRig, Smooth, YawPitch};

// Give camera a component so we can find it and update with Dolly rig
struct MainCamera;

struct PreviewCube;

struct CustomMeshManager {
    pub point_positions: Vec<Vec3>,
    pub mesh_handle: Option<Handle<Mesh>>,
}

impl CustomMeshManager {
    pub fn new() -> Self {
        Self {
            point_positions: Vec::new(),
            mesh_handle: None,
        }
    }

    pub fn to_array(&self) -> Vec<[f32; 3]> {
        let mut one_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1] + 1.0, p[2]], [p[0], p[1], p[2]]])
            .flatten()
            .collect();
        let mut other_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1], p[2]], [p[0], p[1] + 1.0, p[2]]])
            .flatten()
            .collect();
        other_side.reverse();
        //one_side.reverse();
        //out.extend(&one_side);
        //println!("{:?}", out);
        //out
        one_side.extend(&other_side);
        one_side
    }
}

struct CustomMesh;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .insert_resource(CustomMeshManager::new())
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
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    //commands.spawn_scene(asset_server.load("floor.gltf#Scene0"));
    //let floor_handle: Handle<Mesh> = asset_server.load("test.glb#Mesh0/Primitive0");

    let mut bevy_mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    let (gltf, buffers, _) = gltf::import("assets/floor.glb").unwrap();
    for mesh in gltf.meshes() {
        println!("Mesh #{}", mesh.index());
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(vertex_attribute) = reader.read_colors(0).map(|v| {
                bevy::render::mesh::VertexAttributeValues::Float4(v.into_rgba_f32().collect())
            }) {
                println!("ATTRIBUTE_COLOR");
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_positions()
                .map(|v| bevy::render::mesh::VertexAttributeValues::Float3(v.collect()))
            {
                println!("ATTRIBUTE_POSITION");
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_normals()
                .map(|v| bevy::render::mesh::VertexAttributeValues::Float3(v.collect()))
            {
                println!("ATTRIBUTE_NORMAL");
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_tangents()
                .map(|v| bevy::render::mesh::VertexAttributeValues::Float4(v.collect()))
            {
                println!("ATTRIBUTE_TANGENT");
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_TANGENT, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_tex_coords(0)
                .map(|v| bevy::render::mesh::VertexAttributeValues::Float2(v.into_f32().collect()))
            {
                println!("ATTRIBUTE_UV_0");
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vertex_attribute);
            }

            if let Some(indices) = reader.read_indices() {
                bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
                    indices.into_u32().collect(),
                )));
            };

            println!("- Primitive #{}", primitive.index());
            for (semantic, _) in primitive.attributes() {
                println!("-- {:?}", semantic);
            }
        }
    }

    // plane

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(bevy_mesh), //meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: asset_server.load("test.glb#Material0"), //materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());

    // cube
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
    mut custom_mesh_manager: ResMut<CustomMeshManager>,
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
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        custom_mesh_manager.mesh_handle = None;
        custom_mesh_manager.point_positions = Vec::new();
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
        for camera in query.iter_mut() {
            //println!("{:?}", camera.intersect_top());

            if let Some((_, intersection)) = camera.intersect_top() {
                custom_mesh_manager
                    .point_positions
                    .push(intersection.position());

                if custom_mesh_manager.point_positions.len() == 2 {
                    let mut mesh =
                        Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleStrip);

                    let pos = custom_mesh_manager.to_array();
                    let vert_count = pos.len();

                    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; vert_count]);
                    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; vert_count]);
                    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
                        (0..vert_count).map(|i| i as u32).collect(),
                    )));

                    let handle = meshes.add(mesh);
                    custom_mesh_manager.mesh_handle = Some(handle.clone());

                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: handle,
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
                            ..Default::default()
                        })
                        .insert(CustomMesh);
                } else if let Some(mesh_handle) = custom_mesh_manager.mesh_handle.as_ref() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        let pos = custom_mesh_manager.to_array();
                        let vert_count = pos.len();
                        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                        mesh.set_attribute(
                            Mesh::ATTRIBUTE_NORMAL,
                            vec![[1.0, 0.0, 0.0]; vert_count],
                        );
                        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; vert_count]);
                        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
                            (0..vert_count).map(|i| i as u32).collect(),
                        )));
                    }
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
