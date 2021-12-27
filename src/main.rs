use asset_libraries::mesh_library::AssetMeshLibrary;
use asset_libraries::shader_library::AssetShaderLibrary;
use asset_libraries::vao_library::AssetVAOLibrary;

use bevy_app::App;
use bevy_ecs::prelude::*;

use bracket_noise::prelude::FastNoise;
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

// TODO: make the walls more straight at the top, but still adapt to the ground
// maybe make some kind of Y smoothing?

pub struct TerrainData {
    perlin: bracket_noise::prelude::FastNoise,
    pub amp: f32,
    pub offset: glam::Vec2,
    min_y: f32,
    max_y: f32,
    pub texture: u32,
    pub texture_dims: (i32, i32),
}

impl TerrainData {
    pub fn recalculate_texture(&mut self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            let (raw_pixels, min, max) =
                Self::raw_pixels_f32(&self.perlin, self.texture_dims, self.offset, self.amp);

            self.min_y = min;
            self.max_y = max;

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.texture_dims.0,
                self.texture_dims.1,
                gl::RGBA,
                gl::FLOAT,
                &raw_pixels[0] as *const f32 as *const std::ffi::c_void,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        self.perlin.get_noise(self.offset.x + x, self.offset.y + y) * self.amp
    }

    pub fn raw_pixels_f32(
        noise: &FastNoise,
        texture_dims: (i32, i32),
        offset: glam::Vec2,
        amp: f32,
    ) -> (Vec<f32>, f32, f32) {
        let size = (20.0, 20.0);
        let mut raw_pixels = Vec::new();

        let mut min_value = 0.0;
        let mut max_value = 0.0;

        for y in 0..texture_dims.1 {
            let p_y = (y as f32 / texture_dims.1 as f32) * size.1 - size.1 / 2.0;

            for x in 0..texture_dims.0 {
                let p_x = (x as f32 / texture_dims.0 as f32) * size.0 - size.0 / 2.0;

                let n = noise.get_noise(p_x + offset.x, p_y + offset.y) * amp;
                raw_pixels.extend([n, n, n, 1.0]);

                if n < min_value {
                    min_value = n;
                }

                if n > max_value {
                    max_value = n;
                }
            }
        }

        (raw_pixels, min_value, max_value)
    }

    pub fn new() -> Self {
        let mut noise = bracket_noise::prelude::FastNoise::seeded(45);
        noise.set_noise_type(bracket_noise::prelude::NoiseType::PerlinFractal);
        noise.set_fractal_type(bracket_noise::prelude::FractalType::FBM);
        noise.set_fractal_octaves(3);
        noise.set_fractal_gain(1.0);
        noise.set_fractal_lacunarity(3.0);
        noise.set_frequency(0.05);
        noise.set_seed(0);

        let amp = 1.3;
        let offset = glam::Vec2::ZERO;
        let texture_dims = (512, 512);

        // generate texture
        let min_y;
        let max_y;
        let texture = unsafe {
            let (raw_pixels, min, max) = Self::raw_pixels_f32(&noise, texture_dims, offset, amp);

            min_y = min;
            max_y = max;
            // Create texture
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                texture_dims.0,
                texture_dims.1,
                0,
                gl::RGBA,
                gl::FLOAT,
                &raw_pixels[0] as *const f32 as *const std::ffi::c_void,
            );
            texture
        };

        Self {
            perlin: noise,
            min_y,
            max_y,
            texture,
            amp,
            offset,
            texture_dims,
        }
    }
}

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
        .insert_resource(TerrainData::new())
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
        )
        .add_system_to_stage("main_singlethread", update_terrain.system());

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
