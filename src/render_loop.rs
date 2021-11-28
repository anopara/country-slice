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
use crate::window_events::WindowSize;
use crate::{ComputeTest, CursorRaycast, DisplayTestMask};

use crate::components::{drawable::GLDrawMode, transform::Transform};

pub fn render(ecs: &mut World, windowed_context: &mut ContextWrapper<PossiblyCurrent, Window>) {
    let mut img_unit = 0;

    // render
    // ------

    unsafe {
        gl::DepthMask(gl::TRUE);

        let test = ecs.get_resource::<ComputeTest>().unwrap();
        let mouse = ecs.get_resource::<CursorRaycast>().unwrap();
        let mouse_button_input = ecs.get_resource::<Input<MouseButton>>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();

        // COMPUTE SHADER PASS -----------------------------------------------------------------------
        // Only update shader if RMB is pressed
        if mouse_button_input.pressed(MouseButton::Right) {
            let shader = assets_shader.get(test.compute_program).unwrap();

            gl::UseProgram(shader.id());

            // connect shader's uniform variable to our texture
            // instead of name can specify in shader the binding, for ex "layout(rgba32f, binding = 0)"
            let uniform_name = CString::new("img_output").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1ui(tex_location, img_unit);
            // bind texture
            gl::BindImageTexture(
                img_unit,
                test.texture,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );

            shader.set_gl_uniform("Mouse_Position", GlUniform::Vec3(mouse.0.to_array()));
            gl::DispatchCompute(test.texture_dims.0 as u32, test.texture_dims.1 as u32, 1);

            img_unit += 1;
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
        )>();
        let assets_vao = ecs.get_resource::<AssetVAOLibrary>().unwrap();
        let assets_shader = ecs.get_resource::<AssetShaderLibrary>().unwrap();

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

            // MEOWMEOWcheckforspecialtexture
            if test.is_some() {
                gl::BindTexture(gl::TEXTURE_2D, texture_buffer);
            }

            gl::BindVertexArray(vao.id());

            // Set model, view and projection transforms as uniforms
            for (name, transform) in &[
                ("model", model_transform.compute_matrix().to_cols_array()),
                ("view", view_transform.to_cols_array()),
                ("projection", projection_transform.to_cols_array()),
            ] {
                shader.set_gl_uniform(name, GlUniform::Mat4(*transform));
            }

            let mode = gl_draw_flag.map(|c| c.0).unwrap_or(gl::TRIANGLES);
            if let Some(instanced_wall) = instanced_wall {
                // Set wall uniforms
                shader.set_gl_uniform(
                    "camera_position",
                    GlUniform::Vec3(camera_position.to_array()),
                );

                shader.set_gl_uniform("wall_length", GlUniform::Float(instanced_wall.wall_length));

                // bind to shader
                instanced_wall
                    .instance_buffer
                    .bind(shader, "instanced_wall_data");

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

            gl::BindVertexArray(vao.id());

            // Set model, view and projection transforms as uniforms
            for (name, transform) in &[
                ("model", model_transform.compute_matrix().to_cols_array()),
                ("view", view_transform.to_cols_array()),
                ("projection", projection_transform.to_cols_array()),
            ] {
                shader.set_gl_uniform(name, GlUniform::Mat4(*transform));
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
