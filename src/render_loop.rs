use std::ffi::CString;
use std::ptr;

use bevy_ecs::prelude::World;

use bevy_input::mouse::MouseButton;
use bevy_input::Input;

use glutin::{window::Window, ContextWrapper, PossiblyCurrent};

use crate::asset_libraries::{
    shader_library::AssetShaderLibrary, vao_library::AssetVAOLibrary, Handle,
};
use crate::components::drawable::TransparencyPass;
use crate::geometry::instanced_wall::InstancedWall;
use crate::render::{
    camera::MainCamera,
    shader::{GlUniform, ShaderProgram},
    vao::VAO,
};
use crate::systems::draw_wall::WallManager;
use crate::window_events::WindowSize;
use crate::{
    ComputeDrawIndirectTest, ComputeTest, CursorRaycast, DisplayTestMask,
    DrawElementsIndirectCommand, IndirectDraw,
};

use crate::utils::custom_macro::log_if_error;

use crate::components::{drawable::GLDrawMode, transform::Transform};

pub fn render(ecs: &mut World, windowed_context: &mut ContextWrapper<PossiblyCurrent, Window>) {
    let mut _img_unit = 0;

    // render
    // ------

    unsafe {
        gl::DepthMask(gl::TRUE);

        let indirect_test = ecs.get_resource::<ComputeDrawIndirectTest>().unwrap();
        let test = ecs.get_resource::<ComputeTest>().unwrap();
        let wall_manager = ecs.get_resource::<WallManager>().unwrap();
        // INDIRECT COMPUTE SHADER PASS -----------------------------------------------------------------------
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();

        // Reset draw command buffer to its default
        {
            //log::debug!("Resetting draw command buffer...");
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, indirect_test.command_buffer);
            let ptr = gl::MapBuffer(gl::DRAW_INDIRECT_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut DrawElementsIndirectCommand, 1);
            dst.copy_from_slice(&[DrawElementsIndirectCommand {
                _count: 312, // number of vertices of brick.glb
                _instance_count: 0,
                _first_index: 0,
                _base_vertex: 0,
                _base_instance: 0,
            }]);
            gl::UnmapBuffer(gl::DRAW_INDIRECT_BUFFER);
        }

        // For debugging, reset the transform buffer
        {
            //log::debug!("Resetting transform buffer...");
            let data = &[glam::Mat4::IDENTITY; 10000];
            gl::BindBuffer(
                gl::SHADER_STORAGE_BUFFER,
                indirect_test.transforms_buffer.gl_id(),
            );
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut glam::Mat4, data.len());
            dst.copy_from_slice(data);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
        }

        indirect_test.bind(assets_shader, test.texture, _img_unit); // use shader & bind command buffer & bind transforms buffer & bind road mask

        // bind compute road texture
        gl::DispatchCompute(wall_manager.curves.len() as u32, 1, 1);
        gl::MemoryBarrier(gl::COMMAND_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT);

        // COMPUTE SHADER PASS -----------------------------------------------------------------------

        let test = ecs.get_resource::<ComputeTest>().unwrap();
        let mouse = ecs.get_resource::<CursorRaycast>().unwrap();
        let mouse_button_input = ecs.get_resource::<Input<MouseButton>>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();
        // Only update shader if RMB is pressed
        if mouse_button_input.pressed(MouseButton::Right) {
            let shader = assets_shader.get(test.compute_program).unwrap();

            gl::UseProgram(shader.id());

            // connect shader's uniform variable to our texture
            // instead of name can specify in shader the binding, for ex "layout(rgba32f, binding = 0)"
            let uniform_name = CString::new("img_output").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1ui(tex_location, _img_unit);
            // bind texture
            gl::BindImageTexture(
                _img_unit,
                test.texture,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );

            log_if_error!(
                shader.set_gl_uniform("Mouse_Position", GlUniform::Vec3(mouse.0.to_array()))
            );
            gl::DispatchCompute(test.texture_dims.0 as u32, test.texture_dims.1 as u32, 1);

            _img_unit += 1;
        }

        // make sure writing to image has finished before read
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        let texture_buffer = test.texture;

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
            Option<&crate::RoadComponent>,
        )>();
        let assets_vao = ecs.get_resource::<AssetVAOLibrary>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();
        let indirect_test = ecs.get_resource::<ComputeDrawIndirectTest>().unwrap();

        let mut transparent_pass = Vec::new();

        // --------------------------
        for (
            vao_handle,
            shader_handle,
            model_transform,
            gl_draw_flag,
            instanced_wall,
            test,
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

            // MEOWMEOWcheckforspecialtexture
            if test.is_some() {
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer);
            }

            // check if its a road
            if road.is_some() {
                // bind road mask
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer);
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
