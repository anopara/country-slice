use std::ffi::CString;
use std::ptr;

use bevy_ecs::prelude::World;

use bevy_input::mouse::MouseButton;
use bevy_input::Input;

use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

use crate::asset_libraries::{
    shader_library::AssetShaderLibrary, vao_library::AssetVAOLibrary, Handle,
};
use crate::geometry::instanced_wall::InstancedWall;
use crate::render::{
    camera::MainCamera,
    shader::{GlUniform, ShaderProgram},
    vao::VAO,
};
use crate::resources::compute_path_mask::*;
use crate::resources::curve_segments_pass::CURVE_BUFFER_SIZE;
use crate::resources::CurveSegmentsComputePass;
use crate::systems::mode_manager::{BrushMode, EraseLayer};
use crate::window_events::WindowSize;
use crate::{components::*, TerrainData};
use crate::{ComputeArchesIndirect, ComputePathMask, CursorRaycast};

use crate::utils::custom_macro::log_if_error;

use crate::components::{drawable::GLDrawMode, transform::Transform};

pub fn render(ecs: &mut World, windowed_context: &mut ContextWrapper<PossiblyCurrent, Window>) {
    puffin::profile_function!();

    let mut _img_unit = 0;

    // render
    // ------

    unsafe {
        gl::DepthMask(gl::TRUE);

        let indirect_test = ecs.get_resource::<ComputeArchesIndirect>().unwrap();
        let compute_curve_segments = ecs.get_resource::<CurveSegmentsComputePass>().unwrap();
        let path_mask = &ecs.get_resource::<ComputePathBlur>().unwrap().0;
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();

        // CURVE SEGMNETS COMPUTE
        {
            compute_curve_segments.reset_cmd_buffer();
            compute_curve_segments.reset_segments_buffer();
            compute_curve_segments.bind(
                assets_shader,
                path_mask.texture.id,
                PATH_MASK_WS_DIMS,
                _img_unit,
            );

            gl::DispatchCompute(CURVE_BUFFER_SIZE as u32, 1, 1);
            gl::MemoryBarrier(gl::COMMAND_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT);
        }

        // INDIRECT COMPUTE SHADER PASS -----------------------------------------------------------------------

        indirect_test.reset_draw_command_buffer();
        indirect_test.reset_transform_buffer();

        indirect_test.bind(
            assets_shader,
            &compute_curve_segments.segments_buffer,
            path_mask.texture.id,
            PATH_MASK_WS_DIMS,
            _img_unit,
        ); // use shader & bind command buffer & bind transforms buffer & bind road mask

        // bind compute road texture
        gl::DispatchComputeIndirect(0);
        gl::MemoryBarrier(gl::COMMAND_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT);

        // COMPUTE PATHS PASS -----------------------------------------------------------------------

        let path_mask = &ecs.get_resource::<ComputePathMask>().unwrap().0;
        let path_blur = &ecs.get_resource::<ComputePathBlur>().unwrap().0;
        let mouse = ecs.get_resource::<CursorRaycast>().unwrap();
        let mouse_button_input = ecs.get_resource::<Input<MouseButton>>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();
        let _mode = ecs.get_resource::<BrushMode>().unwrap();
        // Only update shader if LMB is pressed and we are in Path mode

        if (matches!(_mode, BrushMode::Path) || matches!(_mode, BrushMode::Eraser(EraseLayer::All)))
            && mouse_button_input.pressed(MouseButton::Left)
        {
            let shader = assets_shader.get(path_mask.compute_program).unwrap();
            gl::UseProgram(shader.id());

            match _mode {
                BrushMode::Wall => panic!(),
                BrushMode::Path => {
                    log_if_error!(shader.set_gl_uniform("is_additive", GlUniform::Bool(true)))
                }
                BrushMode::Eraser(..) => {
                    log_if_error!(shader.set_gl_uniform("is_additive", GlUniform::Bool(false)))
                }
            }

            // connect shader's uniform variable to our texture
            // instead of name can specify in shader the binding, for ex "layout(rgba32f, binding = 0)"
            let uniform_name = CString::new("img_output").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1i(tex_location, _img_unit as i32);

            // bind texture
            gl::BindImageTexture(
                _img_unit,
                path_mask.texture.id,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );

            log_if_error!(
                shader.set_gl_uniform("Mouse_Position", GlUniform::Vec3(mouse.0.to_array()))
            );
            log_if_error!(shader.set_gl_uniform("path_mask_ws_dims", GlUniform::Vec2([20.0, 20.0])));
            gl::DispatchCompute(
                path_mask.texture.dims.0 as u32,
                path_mask.texture.dims.1 as u32,
                1,
            );

            _img_unit += 1;
        }

        // make sure writing to image has finished before read
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        // BLUR PATH MASK -------------------------------------

        {
            _img_unit = 0;
            let shader = assets_shader.get(path_blur.compute_program).unwrap();
            gl::UseProgram(shader.id());

            let uniform_name = CString::new("img_in").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1i(tex_location, _img_unit as i32);

            // bind texture
            gl::BindImageTexture(
                _img_unit,
                path_mask.texture.id,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );
            _img_unit += 1;

            let uniform_name = CString::new("img_out").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1i(tex_location, _img_unit as i32);

            // bind texture
            gl::BindImageTexture(
                _img_unit,
                path_blur.texture.id,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );
            _img_unit += 1;

            gl::DispatchCompute(
                path_mask.texture.dims.0 as u32,
                path_mask.texture.dims.1 as u32,
                1,
            );

            // make sure writing to image has finished before read
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        // -----------------------------------------------------------------------

        let texture_buffer = path_blur.texture.id; //path_mask.texture.id;
        let texture_buffer_blur = path_blur.texture.id;

        // MAIN PASS --------------------------------------------------------------------------------

        let (width, height) = ecs.get_resource::<WindowSize>().unwrap().try_into_i32();
        gl::Viewport(0, 0, width, height);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // back to default

        gl::ClearColor(0.120741, 0.120741, 0.120741, 1.0); // same as the floor.glb edges
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let main_camera = ecs.get_resource::<MainCamera>().unwrap();

        let view_transform = main_camera.camera.world_to_camera_view();
        let camera_position = main_camera.camera.position();
        let projection_transform = main_camera.camera.perspective_projection;

        // Gather all drawable entities
        let mut query = ecs.query::<(
            &Handle<VAO>,
            &Handle<ShaderProgram>,
            &Transform,
            Option<&GLDrawMode>,
            Option<&InstancedWall>,
            Option<&DisplayTestMask>,
            Option<&TransparencyPass>,
            Option<&IndirectDraw>,
            Option<&RoadComponent>,
        )>();
        let assets_vao = ecs.get_resource::<AssetVAOLibrary>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();
        let indirect_test = ecs.get_resource::<ComputeArchesIndirect>().unwrap();
        let terrain_data = ecs.get_resource::<TerrainData>().unwrap();

        let mut transparent_pass = Vec::new();

        // --------------------------
        for (
            vao_handle,
            shader_handle,
            model_transform,
            gl_draw_flag,
            instanced_wall,
            debug_display_path_mask,
            transparency,
            indirect_draw,
            road,
        ) in query.iter(ecs)
        {
            let vao = assets_vao
                .get(*vao_handle)
                .expect("Oops! This VAO handle is invalid");

            let shader = assets_shader
                .get(*shader_handle)
                .expect("Oops! This Shader handle is invalid");

            if transparency.is_some() {
                // skip rendering transparent objects
                // stash them for later
                transparent_pass.push((vao, shader, model_transform));

                continue;
            }

            // Render
            shader.gl_use_program();
            //println!(
            //    "Using shader with name {:?}",
            //    assets_shader.debug_get_name(*shader_handle)
            //);

            {
                //DEBUG TERRAIN TEXTURE
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, terrain_data.texture.id);
                shader.set_gl_uniform("terrain_texture", GlUniform::Int(1));
                //reset
                gl::ActiveTexture(gl::TEXTURE0);
            }

            // MEOWMEOWcheckforspecialtexture
            if debug_display_path_mask.is_some() {
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer_blur);
            }

            // check if its a road
            if road.is_some() {
                // bind road mask
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer);
                log_if_error!(shader.set_gl_uniform("path_texture", GlUniform::Int(0)));

                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, terrain_data.texture.id);
                log_if_error!(shader.set_gl_uniform("terrain_texture", GlUniform::Int(1)));
                //reset
                gl::ActiveTexture(gl::TEXTURE0);
                // TODO: when we used the `instanced_wall.frag` for shading, re-enable this
                //shader.set_gl_uniform("is_arch", GlUniform::Bool(true));
            }

            gl::BindVertexArray(vao.id());

            // Set model, view and projection transforms as uniforms
            for (name, transform) in &[
                ("model", model_transform.compute_matrix().to_cols_array()),
                ("view", view_transform.to_cols_array()),
                ("projection", projection_transform.to_cols_array()),
            ] {
                // it's OK if the shader is not using one of these uniforms, that's not an error
                let _result = shader.set_gl_uniform(name, GlUniform::Mat4(*transform));
            }

            // check if its an indirect draw
            if indirect_draw.is_some() {
                // used for disabling discarding of fragments
                log_if_error!(shader.set_gl_uniform("is_arch", GlUniform::Bool(true)));
                indirect_test
                    .transforms_buffer
                    .bind(&shader, "transforms_buffer");
                gl::DrawElementsIndirect(gl::TRIANGLES, gl::UNSIGNED_INT, ptr::null());

                continue;
            }

            let mode = gl_draw_flag.map(|c| c.0).unwrap_or(gl::TRIANGLES);
            if let Some(instanced_wall) = instanced_wall {
                // Set wall uniforms
                log_if_error!(shader.set_gl_uniform(
                    "camera_position",
                    GlUniform::Vec3(camera_position.to_array()),
                ));

                log_if_error!(shader
                    .set_gl_uniform("wall_length", GlUniform::Float(instanced_wall.wall_length)));

                // used for disabling discarding of fragments
                log_if_error!(shader.set_gl_uniform("is_arch", GlUniform::Bool(false)));

                // bind to shader
                instanced_wall
                    .instance_buffer
                    .bind(shader, "instanced_wall_data");

                // bind compute shader texture
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer);

                gl::DrawArraysInstanced(
                    mode,
                    0,
                    vao.indices_count as i32,
                    instanced_wall.instance_buffer.instance_num as i32,
                );
            } else {
                // The second argument is the count or number of elements(aka indices to draw)
                // the last argument allows us to specify an offset in the EBO
                gl::DrawElements(
                    mode,
                    vao.indices_count as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }

        // TRANSPARENCY PASS ---------------------------------------------------------
        gl::DepthMask(gl::FALSE);
        for (vao, shader, model_transform) in transparent_pass {
            // Render
            shader.gl_use_program();

            {
                //DEBUG TERRAIN TEXTURE
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, terrain_data.texture.id);
                log_if_error!(shader.set_gl_uniform("terrain_texture", GlUniform::Int(1)));
                //reset
                gl::ActiveTexture(gl::TEXTURE0);
            }

            // atm, I'm just binding the road mask to anything in transparency pass (ATM, only shadows have transparency pass, so we can just bind the texture)
            // TODO: in the future, need to check for whether its a shadow
            gl::BindTexture(gl::TEXTURE_2D, texture_buffer);

            gl::BindVertexArray(vao.id());

            // Set model, view and projection transforms as uniforms
            for (name, transform) in &[
                ("model", model_transform.compute_matrix().to_cols_array()),
                ("view", view_transform.to_cols_array()),
                ("projection", projection_transform.to_cols_array()),
            ] {
                let _result = shader.set_gl_uniform(name, GlUniform::Mat4(*transform));
            }

            gl::DrawElements(
                gl::TRIANGLES,
                vao.indices_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
    windowed_context.swap_buffers().unwrap();
}
