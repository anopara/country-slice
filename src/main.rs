use asset_libraries::mesh_library::AssetMeshLibrary;
use asset_libraries::shader_library::AssetShaderLibrary;
use asset_libraries::vao_library::AssetVAOLibrary;
use asset_libraries::Handle;

use bevy_app::App;
use bevy_ecs::prelude::*;

use gl::types::GLsizeiptr;
use glam::Vec3;
use glutin::event_loop::ControlFlow;

use render::{camera::MainCamera, ssbo::GLShaderStorageBuffer};

use render::shader::ShaderProgram;
use render::shaderwatch::*;
use window_events::{process_window_events, CursorMoved, WindowSize};

use crate::systems::*;

mod asset_libraries;
mod components;
mod geometry;
mod render;
mod render_loop;
mod setup;
mod systems;
mod utils;
mod window_events;

// https://github.com/bwasty/learn-opengl-rs
// https://learnopengl.com/Getting-started/Hello-Triangle

// settings
const SCR_WIDTH: u32 = 1600;
const SCR_HEIGHT: u32 = 1200;

// Mark the cube that is the preview of mouse raycast intersection
pub struct MousePreviewCube;

pub struct CursorRaycast(pub Vec3);

pub struct DisplayTestMask;

// 1. create a compute shader
// 2. it will need an SSBO where I store wall curves (probably easier to create a custom SSBO struct...)
// 3. this shader will need to read the road sdf texture
// 4. this shader will need logic of arranging the bricks into arch shape
// 4. this shader will need to output draw commands for bricks to be drawn with `glDrawArraysIndirect` (see https://lingtorp.com/2018/12/05/OpenGL-SSBO-indirect-drawing.html)

// TODO: make the walls realistic size.. atm wall height is 1.4m that's very low & arches look out of proportion

// TODO: rename to ComputeRoadMask
struct ComputeTest {
    compute_program: Handle<ShaderProgram>,
    texture: u32,
    texture_dims: (i32, i32),
}

// COMPUTE SHADER INDIRECT DRAW
pub struct ComputeDrawIndirectTest {
    compute_program: Handle<ShaderProgram>,
    command_buffer: u32,
    command_buffer_binding_point: u32,
    //
    pub transforms_buffer: GLShaderStorageBuffer<glam::Mat4>,
    //
    pub curves_buffer: GLShaderStorageBuffer<CurveDataSSBO>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CurveDataSSBO {
    points_count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    positions: [[f32; 4]; 1000], //buffer
}

impl CurveDataSSBO {
    pub fn from(curve: &geometry::curve::Curve) -> Self {
        let points_count = curve.points.len() as u32;
        let mut positions = [[0.0; 4]; 1000];

        positions.iter_mut().enumerate().for_each(|(i, p)| {
            *p = curve
                .points
                .get(i)
                .unwrap_or(&Vec3::ZERO)
                .extend(1.0)
                .to_array()
        });

        Self {
            points_count,
            positions,
            pad0: 0,
            pad1: 0,
            pad2: 0,
        }
    }
}

impl ComputeDrawIndirectTest {
    pub fn bind(
        &self,
        assets_shader: &AssetShaderLibrary,
        road_mask: u32,
        road_mask_img_unit: u32,
    ) {
        unsafe {
            // bind compute shader
            let shader = assets_shader.get(self.compute_program).unwrap();
            gl::UseProgram(shader.id());

            // bind command buffer
            //from: https://lingtorp.com/2018/12/05/OpenGL-SSBO-indirect-drawing.html

            let c_str = std::ffi::CString::new("draw_commands").unwrap();
            let block_index = gl::GetProgramResourceIndex(
                shader.id(),
                gl::SHADER_STORAGE_BLOCK,
                c_str.as_ptr() as *const std::os::raw::c_char,
            );
            gl::ShaderStorageBlockBinding(
                shader.id(),
                block_index,
                self.command_buffer_binding_point,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.command_buffer);
            gl::BindBufferBase(
                gl::SHADER_STORAGE_BUFFER,
                self.command_buffer_binding_point,
                self.command_buffer,
            );

            // bind transforms buffer
            self.transforms_buffer.bind(&shader, "transforms_buffer");

            // bind road mask
            let uniform_name = std::ffi::CString::new("road_mask").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1ui(tex_location, road_mask_img_unit);
            // bind texture
            gl::BindImageTexture(
                road_mask_img_unit,
                road_mask,
                0,
                gl::FALSE,
                0,
                gl::READ_ONLY,
                gl::RGBA32F,
            );

            // bind curve ssbo
            self.curves_buffer.bind(shader, "curves_buffer");
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct DrawElementsIndirectCommand {
    _count: u32,
    _instance_count: u32,
    _first_index: u32,
    _base_vertex: u32,
    _base_instance: u32,
}

// component
struct IndirectDraw;

//
struct RoadComponent;

const COMMAND_BUFFER_SIZE: usize = 1000;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let (mut windowed_context, event_loop) =
        setup::setup_glutin_and_opengl((SCR_WIDTH, SCR_HEIGHT));

    let mut temp_shaderwatch = ShaderWatch::new();
    let mut temp_assets_shader = AssetShaderLibrary::new();
    /*
    // COMPUTE SHADER INDIRECT DRAW  -------------------------------------------
    let compute_indirect_test = unsafe {
        // create shader program
        let shader_program = ShaderProgram::new_compute("shaders/indirect_draw_test.glsl").unwrap();
        temp_shaderwatch.watch(&shader_program);
        let handle = temp_assets_shader.add(shader_program.into());

        // Setup GL_DRAW_INDIRECT_BUFFER for indirect drawing (basically a command buffer)
        let mut ibo = 0;
        gl::GenBuffers(1, &mut ibo);
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, ibo);
        // Unlike `gl::BufferData`, `glBufferStorage` don't allow you to de-allocate it
        // that means the buffer is persistented in the memory, and you don't need to unmap until you really don't need it
        // (c) https://stackoverflow.com/questions/27810542/what-is-the-difference-between-glbufferstorage-and-glbufferdata
        gl::BufferStorage(
            gl::DRAW_INDIRECT_BUFFER,
            (std::mem::size_of::<DrawElementsIndirectCommand>() * COMMAND_BUFFER_SIZE)
                as GLsizeiptr,
            std::ptr::null(),
            gl::MAP_READ_BIT | gl::MAP_WRITE_BIT, // do I need write here to if I'm to write into that storage?
        );

        ComputeDrawIndirectTest {
            compute_program: handle,
            command_buffer: ibo,
            command_buffer_binding_point: 0,
            transforms_buffer: GLShaderStorageBuffer::<glam::Mat4>::new(&vec![], 10000, 2),
            curves_buffer: GLShaderStorageBuffer::<CurveDataSSBO>::new(&vec![], 1000, 3),
        }
    };

    // -----------------------------------------------------------

    // COMPUTE SHADER -------------------------------------------
    let compute_test = unsafe {
        let texture_dims = (512, 512);
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
            std::ptr::null(),
        );
        // create shader program
        let shader_program = ShaderProgram::new_compute("shaders/compute_test.glsl").unwrap();

        temp_shaderwatch.watch(&shader_program);
        let handle = temp_assets_shader.add(shader_program.into());

        ComputeTest {
            compute_program: handle,
            texture,
            texture_dims,
        }
    };

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
        .insert_resource(compute_test) //TODO: Rename
        .insert_resource(compute_indirect_test) // TEST
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
        */

    let mut app = App::build();
    app.add_event::<CursorMoved>() // add these events, to avoid loading the whole bevy_window plugin
        .insert_resource(WindowSize::new(SCR_WIDTH, SCR_HEIGHT))
        .insert_resource(MainCamera::new(SCR_WIDTH as f32 / SCR_HEIGHT as f32))
        .insert_resource(temp_shaderwatch)
        .insert_resource(AssetMeshLibrary::new())
        .insert_resource(AssetVAOLibrary::new())
        .insert_resource(temp_assets_shader);
    // main loop
    // -----------
    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't dispatched any events
        *control_flow = ControlFlow::Poll;

        app.app.update();

        process_window_events(
            event,
            &mut windowed_context,
            control_flow,
            &mut app.world_mut(),
        );
    });
}
