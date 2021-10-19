// TODO: Part 1. SETUP
// 1. DONE setup Bevy renderer
// 2. DONE import a mesh
// 3. DONE render the mesh
// 4. DONE setup the camera with dolly
// 5. DONE draw a SOMETHING with the mouse

//----
// 1. for funs, fix normals (make a gif to record progress)
// 2. make a prototype in Houdini of the wall setup - make an implementation plan

mod utils;

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

use dolly::prelude::{Arm, CameraRig, Smooth, YawPitch};
use itertools::Itertools;

// Give camera a component so we can find it and update with Dolly rig
struct MainCamera;

// Mark the cube that is the preview of mouse raycast intersection
struct PreviewCube;

// TEMPORARY, needs proper mesh data structure for the wall base
// can be intersting? https://crates.io/crates/tri-mesh
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

    pub fn smooth_positions(&self) -> Vec<Vec3> {
        let points_per_segment = 10;
        let smoothing_steps = 50;

        // resample curve
        let mut resampled: Vec<Vec3> = Vec::new();
        for (i, current_pos) in self.point_positions.iter().enumerate() {
            if let Some(next_pos) = self.point_positions.get(i + 1) {
                let dir = *next_pos - *current_pos;
                resampled.extend(
                    &(0..points_per_segment)
                        .map(|s| *current_pos + dir * (s as f32 / points_per_segment as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                // if last point, just add
                resampled.push(*current_pos);
            }
        }

        // smooth
        let mut total_smoothed = resampled.clone();
        for _ in 0..smoothing_steps {
            let mut current_iter_smooth = total_smoothed.clone();
            for (i, current_pos) in total_smoothed.iter().enumerate() {
                if let (Some(prev_pos), Some(next_pos)) =
                    (total_smoothed.get(i - 1), total_smoothed.get(i + 1))
                {
                    let avg: Vec3 = (*prev_pos + *next_pos) / 2.0;
                    current_iter_smooth[i] = *current_pos + (avg - *current_pos) * 0.5;
                }
            }
            total_smoothed = current_iter_smooth;
        }

        total_smoothed
    }

    /*
    fn to_vertices(&self) -> Vec<[f32; 3]> {
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
        one_side.extend(&other_side);
        one_side
    }
    */

    fn to_trimesh(&self) -> tri_mesh::mesh::Mesh {
        let curve_positions = self.smooth_positions();
        println!(
            "Smoothed {} points to {}",
            self.point_positions.len(),
            curve_positions.len()
        );

        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<f64> = Vec::new();
        for quad_index in 0..(curve_positions.len() - 1) {
            let start_point = curve_positions[quad_index];
            let end_point = curve_positions[quad_index + 1];

            let vert_index_start = (positions.len() / 3) as u32;

            positions.extend(&vec![
                // start vertex
                start_point[0] as f64,
                start_point[1] as f64,
                start_point[2] as f64,
                // offset up
                start_point[0] as f64,
                start_point[1] as f64 + 1.0,
                start_point[2] as f64,
                // end vertex
                end_point[0] as f64,
                end_point[1] as f64,
                end_point[2] as f64,
                // offset up
                end_point[0] as f64,
                end_point[1] as f64 + 1.0,
                end_point[2] as f64,
            ]);

            indices.extend(
                &([0, 1, 2, 1, 3, 2]
                    .iter()
                    .map(|i| i + vert_index_start)
                    .collect::<Vec<_>>()),
            )
        }

        /*
        // Construct a mesh from indices and positions buffers.
        let mut indices: Vec<_> = (0..(self.point_positions.len() - 1) * 2)
            .map(|i| {
                let mut ind = vec![i as u32, (i + 1) as u32, (i + 2) as u32];
                if i % 2 != 0 {
                    ind.reverse()
                };
                ind
            })
            .flatten()
            .collect();
        let mut other_side = indices.clone();
        other_side.reverse();
        //indices.extend(&other_side);

        let positions = self
            .point_positions
            .iter()
            .map(|p| {
                vec![
                    // original vertex
                    p[0] as f64,
                    p[1] as f64,
                    p[2] as f64,
                    // offset up
                    p[0] as f64,
                    p[1] as f64 + 1.0,
                    p[2] as f64,
                ]
            })
            .flatten()
            .collect();
            */

        println!("-----Building mesh: ");
        println!("indices: {}", indices.len());
        //println!("positions: {:?}", positions);

        let mesh = tri_mesh::MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        println!("-----Done");

        mesh
    }

    pub fn populate_bevy_mesh(&self, bevy_mesh: &mut Mesh) {
        //let vert_pos = self.to_vertices();
        //let vert_count = vert_pos.len();
        let tri_mesh = self.to_trimesh();
        let vert_count = tri_mesh.vertex_iter().count();

        let positions: Vec<[f32; 3]> = tri_mesh
            .positions_buffer_f32()
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let normals: Vec<[f32; 3]> = tri_mesh
            .normals_buffer_f32()
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let mut indices = tri_mesh.indices_buffer();
        let mut other_side = indices.clone();
        other_side.reverse();
        indices.extend(&other_side);

        //println!("indices {:?}", indices);
        //println!("normals {:?}", normals);
        //println!("positions {:?}", positions);

        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals); //vec![[1.0, 0.0, 0.0]; vert_count]);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; vert_count]);
        bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
            indices, //(0..vert_count).map(|i| i as u32).collect(),
        )));
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
            if let Some((_, intersection)) = camera.intersect_top() {
                custom_mesh_manager
                    .point_positions
                    .push(intersection.position());

                if custom_mesh_manager.point_positions.len() == 2 {
                    let mut mesh =
                        Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

                    custom_mesh_manager.populate_bevy_mesh(&mut mesh);
                    let handle = meshes.add(mesh);
                    custom_mesh_manager.mesh_handle = Some(handle.clone());

                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: handle,
                            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                            ..Default::default()
                        })
                        .insert(CustomMesh);
                } else if let Some(mesh_handle) = custom_mesh_manager.mesh_handle.as_ref() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        custom_mesh_manager.populate_bevy_mesh(mesh);
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
