// I dont' want the wall to jitter, so the splits need to be length invariant, as if you are revealing the splits as you draw
// RMB -> adds Ivy (if not near wall, adds a plant?) - or maybe ivy grows together with the wall, but you can also add more? (with a vegetation brush?)

// Maybe its a small story about a knight who ventured into the ruins (you make walls and setdressing)
// then he saw a house (you make an outline an its a house)
// he entered the houset to kill a witch
// everything soaks red from the house, the knight leaves
// dark creatures gather
// the girl comes out of the house and she turns the knight into one of the creatures (you can draw the creatures?)

// Blob shadows!
// and bulging out terrain where you create walls

// IDEA:
// you can double click on the brick, and it will fall off (physically). This way you can create half-destroyed walls!
// from a tech perspective, I can detect where you clicked, and make that brick into its own mesh & entity and enable physics sim. (and if I will have ivy and plants, that will destoy plants!)

mod curve;
mod curve_manager;
mod instanced_wall;
mod shadow_decal;
mod utils;
mod wall_constructor;

use bevy::{
    prelude::*,
    render::{
        mesh::shape,
        pipeline::{
            CompareFunction, DepthBiasState, DepthStencilState, PipelineDescriptor, RenderPipeline,
            StencilFaceState, StencilState,
        },
        render_graph::{base, RenderGraph, RenderResourcesNode},
        shader::ShaderStages,
    },
};
use bevy_dolly::Transform2Bevy;
use bevy_mod_picking::{PickableBundle, PickingCamera, PickingCameraBundle, PickingPlugin};
use dolly::prelude::{Arm, CameraRig, Smooth, YawPitch};

use bevy::render::{
    pipeline::{BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite},
    shader::Shader,
    texture::TextureFormat,
};

use curve::Curve;
use curve_manager::{CurveManager, UserDrawnCurve};
use wall_constructor::WallConstructor;

use bevy::{reflect::TypeUuid, render::renderer::RenderResources};
use instanced_wall::InstancedWall;
use shadow_decal::ShadowDecal;

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "93fb26fc-6c05-489b-9029-601edf703b6b"]
pub struct TimeUniform {
    pub value: f32,
}

const CURVE_SHOW_DEBUG: bool = false;

// Give camera a component so we can find it and update with Dolly rig
struct MainCamera;

// Mark the cube that is the preview of mouse raycast intersection
struct PreviewCube;

struct CustomMesh;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .insert_resource(CurveManager::new())
        .add_startup_system(setup.system())
        .add_system(update_camera.system())
        //.add_system(handle_mouse_clicks.system())
        .add_system(mouse_preview.system())
        .add_system(update_curve_manager.system().label("curve manager"))
        .add_system(update_wall_2.system().after("curve manager").label("wall"))
        //.add_system(animate_shader.system()) //.after("wall"))
        .run();
}

/*
/// In this system we query for the `TimeComponent` and global `Time` resource, and set
/// `time.seconds_since_startup()` as the `value` of the `TimeComponent`. This value will be
/// accessed by the fragment shader and used to animate the shader.
fn animate_shader(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    for mut time_uniform in query.iter_mut() {
        time_uniform.value = time.seconds_since_startup() as f32;
    }
}
*/

fn update_wall_2(
    mut commands: Commands,
    mut curve_manager: ResMut<CurveManager>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let user_curves_count = curve_manager.user_curves.len();
    let wall_pipeline_handle = curve_manager.wall_pipeline_handle.clone().unwrap();
    let shadow_pipeline_handle = curve_manager.shadow_pipeline_handle.clone().unwrap();
    // If there is a curve being drawn
    if let Some(curve) = curve_manager.user_curves.last() {
        if curve.points.len() < 2 {
            return;
        }

        // Calculate brick transforms
        let curve = Curve::from(utils::smooth_points(&curve.points, 50));
        let bricks = WallConstructor::from_curve(&curve);

        // Check if there is already shadow constructed
        if let Some(shadow) = curve_manager.shadow_decals.get_mut(user_curves_count - 1) {
            shadow.update(&curve, &mut meshes);
        } else {
            curve_manager.shadow_decals.push(ShadowDecal::new(
                &curve,
                &mut meshes,
                shadow_pipeline_handle,
                &mut commands,
            ));
        }

        // Check if there is already wall constructed
        if let Some(wall) = curve_manager.instanced_walls.get_mut(user_curves_count - 1) {
            wall.update(bricks, meshes);
        } else {
            curve_manager.instanced_walls.push(InstancedWall::new(
                bricks,
                meshes,
                materials,
                wall_pipeline_handle,
                commands,
            ));
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
    asset_server: Res<AssetServer>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Watch for changes
    asset_server.watch_for_changes().unwrap();

    curve_manager.curve_pipeline_handle = Some(pipelines.add(PipelineDescriptor::default_config(
        ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/curve_test.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/curve_test.frag")),
        },
    )));

    curve_manager.wall_pipeline_handle = Some(pipelines.add(PipelineDescriptor::default_config(
        ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/pbr.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/pbr.frag")),
        },
    )));

    // Same as in `build_pbr_pipeline` but with depth_write_enabled=false, because shadows are transparent
    let shadow_pipeline_descriptor = PipelineDescriptor {
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Less,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
            clamp_depth: false,
        }),
        color_target_states: vec![ColorTargetState {
            format: TextureFormat::default(),
            color_blend: BlendState {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendState {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/shadow.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/shadow.frag")),
        })
    };

    curve_manager.shadow_pipeline_handle = Some(pipelines.add(shadow_pipeline_descriptor));

    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/vertex_color.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/vertex_color.frag")),
    }));

    /*
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        light: Light {
            color: Color::rgb(1.0, 0.0, 0.0),
            ..Default::default()
        },
        global_transform: Default::default(),
    });
    */

    // Add a `RenderResourcesNode` to our `RenderGraph`. This will bind `TimeComponent` to our
    // shader.
    render_graph.add_system_node(
        "time_uniform",
        RenderResourcesNode::<TimeUniform>::new(true),
    );

    // Add a `RenderGraph` edge connecting our new "time_component" node to the main pass node. This
    // ensures that "time_component" runs before the main pass.
    render_graph
        .add_node_edge("time_uniform", base::node::MAIN_PASS)
        .unwrap();

    // floor
    let floor_bundle = PbrBundle {
        mesh: meshes.add(utils::load_gltf_as_bevy_mesh_w_vertex_color(
            "assets/meshes/floor.glb",
        )),
        material: materials.add(Color::WHITE.into()),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        ..Default::default()
    };
    commands
        .spawn_bundle(floor_bundle)
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
    // TODO: can replace this with a resource and update camera
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

/*
fn handle_mouse_clicks(mouse_input: Res<Input<MouseButton>>, windows: Res<Windows>) {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        //println!("click at {:?}", win.cursor_position());
    }
}
*/

fn mouse_preview(
    mut query: Query<&mut PickingCamera>,
    mut cube_query: Query<(
        &mut PreviewCube,
        &mut bevy::transform::components::Transform,
    )>,
) {
    for camera in query.iter_mut() {
        if let Some((_, intersection)) = camera.intersect_top() {
            for (_, mut transform) in cube_query.iter_mut() {
                transform.translation = intersection.position();
            }
        }
    }
}

fn update_curve_manager(
    materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut curve_manager: ResMut<CurveManager>,
    mouse_button_input: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut PickingCamera>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        curve_manager.clear_all(&mut commands);
    }

    // If LMB was just pressed, start a new curve
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let pipeline = curve_manager.curve_pipeline_handle.clone().unwrap();
        curve_manager
            .user_curves
            .push(UserDrawnCurve::new(pipeline));
    }

    // If there is a curve being drawn
    if let Some(curve) = curve_manager.user_curves.last_mut() {
        // Add points to it
        if mouse_button_input.pressed(MouseButton::Left) {
            if let Ok(Some((_, intersection))) =
                query.single_mut().map(|camera| camera.intersect_top())
            {
                const DIST_THRESHOLD: f32 = 0.001;

                if curve
                    .points
                    .last()
                    // if curve  had points, only add if the distance is larger than X
                    .map(|last| intersection.position().distance(*last) > DIST_THRESHOLD)
                    // if curve  has no points, add this point
                    .unwrap_or(true)
                {
                    curve.points.push(intersection.position())
                }
            }
        }

        // Update its debug mesh
        if CURVE_SHOW_DEBUG {
            curve.update_debug_mesh(meshes, materials, commands);
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

    if keys.pressed(KeyCode::Left) {
        camera_driver.rotate_yaw_pitch(-2.0, 0.0);
    }
    if keys.pressed(KeyCode::Right) {
        camera_driver.rotate_yaw_pitch(2.0, 0.0);
    }

    if keys.pressed(KeyCode::Up) {
        camera_driver.rotate_yaw_pitch(0.0, -1.0);
    }
    if keys.pressed(KeyCode::Down) {
        camera_driver.rotate_yaw_pitch(0.0, 1.0);
    }

    let transform = rig.update(time.delta_seconds());
    let (mut cam, _) = query.q0_mut().single_mut().unwrap();

    cam.transform_2_bevy(transform);
}
