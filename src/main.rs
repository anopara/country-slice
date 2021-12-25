use asset_libraries::mesh_library::AssetMeshLibrary;
use asset_libraries::shader_library::AssetShaderLibrary;
use asset_libraries::vao_library::AssetVAOLibrary;

use bevy_app::App;
use bevy_ecs::prelude::*;

use components::CursorRaycast;
use glam::Vec3;
use glutin::event_loop::ControlFlow;

use render::camera::MainCamera;

use render::shaderwatch::*;
use resources::{ComputeArchesIndirect, ComputePathsMask, CurveSegmentsComputePass, WallManager};
use window_events::{process_window_events, CursorMoved, WindowSize};

use crate::systems::*;

mod asset_libraries;
mod components;
mod geometry;
mod render;
mod render_loop;
mod resources;
mod setup;
mod systems;
mod utils;
mod window_events;

// https://github.com/bwasty/learn-opengl-rs
// https://learnopengl.com/Getting-started/Hello-Triangle

// settings
const SCR_WIDTH: u32 = 1600;
const SCR_HEIGHT: u32 = 1200;

const VALIDATE_SHADERS: bool = false;

// TODO: make the walls realistic size.. atm wall height is 1.4m that's very low & arches look out of proportion

// Uneven terrain
// CPU side -> mouse raycast
// * some kind of perlin noise on CPU side
// * then this img needs to be sent to GPU too
// GPU side -> update the meshes, just push the vertices up

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let (mut windowed_context, event_loop) =
        setup::setup_glutin_and_opengl((SCR_WIDTH, SCR_HEIGHT));

    if VALIDATE_SHADERS {
        utils::validate_shaders("shaders/");
    }

    let mut temp_shaderwatch = ShaderWatch::new();
    let mut temp_assets_shader = AssetShaderLibrary::new();

    // COMPUTE SHADERS -------------------------------------------
    let compute_paths_mask = ComputePathsMask::init(&mut temp_shaderwatch, &mut temp_assets_shader);
    let compute_curve_segments =
        CurveSegmentsComputePass::init(&mut temp_shaderwatch, &mut temp_assets_shader);
    let compute_arches_indirect =
        ComputeArchesIndirect::init(&mut temp_shaderwatch, &mut temp_assets_shader);

    // ----------------------------------------------------------

    let mut app = App::build();
    app.add_plugin(bevy_core::CorePlugin::default())
        .add_plugin(bevy_input::InputPlugin::default())
        .add_event::<CursorMoved>() // add these events, to avoid loading the whole bevy_window plugin
        .insert_resource(WindowSize::new(SCR_WIDTH, SCR_HEIGHT))
        .insert_resource(MainCamera::new(SCR_WIDTH as f32 / SCR_HEIGHT as f32))
        .insert_resource(temp_shaderwatch)
        .insert_resource(WallManager::new())
        .insert_resource(CursorRaycast(Vec3::ZERO))
        .insert_resource(AssetMeshLibrary::new())
        .insert_resource(AssetVAOLibrary::new())
        .insert_resource(temp_assets_shader)
        .insert_resource(compute_paths_mask)
        .insert_resource(compute_arches_indirect)
        .insert_resource(compute_curve_segments)
        .add_stage_after(
            bevy_app::CoreStage::PreUpdate,
            "opengl",
            SystemStage::single_threaded(),
        )
        .add_stage_after(
            "opengl",
            "main_singlethread",
            SystemStage::single_threaded(),
        )
        .add_system_to_stage("opengl", shaderwatch.system().label("reload_shaders"))
        .add_system_to_stage("opengl", build_missing_vaos.system().label("build_vaos"))
        .add_system_to_stage("opengl", rebuild_vaos.system().after("build_vaos"))
        //.add_system(draw_curve.system().label("usercurve"))
        .add_system(main_camera_update.system())
        .add_system(mouse_raycast.system())
        .add_system(draw_curve.system().label("usercurve"))
        .add_system_to_stage(
            "main_singlethread",
            update_curve_ssbo.system().after("usercurve"),
        )
        .add_system_to_stage(
            "main_singlethread",
            walls_update.system().after("usercurve"),
        );

    systems::startup(&mut app.world_mut());

    // main loop
    // -----------

    let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
    eprintln!("Serving demo profile data on {}", server_addr);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();

    puffin::set_scopes_on(true);

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't dispatched any events
        *control_flow = ControlFlow::Poll;

        //app.app.update();

        process_window_events(event, &mut windowed_context, control_flow, &mut app);
    });
}
