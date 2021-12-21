use bevy_ecs::{component::Component, prelude::*};

use crate::asset_libraries::Handle;
use crate::components::*;
use crate::geometry::cube::Cube;
use crate::utils::load_json::load_json_as_mesh;

use crate::geometry::plane::Plane;
use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Asset},
    render::{mesh::Mesh, shader::ShaderProgram, shaderwatch::ShaderWatch},
};

pub fn res_mut<T: Component>(ecs: &mut World) -> Mut<'_, T> {
    ecs.get_resource_mut::<T>().unwrap()
}

pub fn startup(ecs: &mut World) {
    puffin::profile_function!();
    // Load meshes
    let floor = load_mesh_into_library(load_mesh("meshes/floor.glb"), "floor", ecs);
    let _brick = load_mesh_into_library(load_mesh("meshes/brick.glb"), "brick", ecs);
    let cube = load_mesh_into_library(Mesh::from(Cube::new(0.1)), "cube", ecs);
    let _plane = load_mesh_into_library(Mesh::from(Plane { size: 20.0 }), "plane", ecs);

    let mut road_pebbles_mesh = load_json_as_mesh("meshes/road_pebbles.json").unwrap();
    road_pebbles_mesh.add_color();
    road_pebbles_mesh.add_uv();
    let road_pebbles = load_mesh_into_library(road_pebbles_mesh, "road", ecs);

    // Load shaders
    let vert_color = load_shader_into_library(
        "shaders/vertex_color.vert",
        "shaders/vertex_color.frag",
        "vertex_color_shader",
        ecs,
    );
    let road_shader = load_shader_into_library(
        "shaders/paths.vert",
        "shaders/vertex_color.frag",
        "road_shader",
        ecs,
    );
    // this shader shows the compute_test.comp as a texture
    let _test = load_shader_into_library(
        "shaders/texture_test.vert",
        "shaders/texture_test.frag",
        "texture_test_shader",
        ecs,
    );
    load_shader_into_library(
        "shaders/instanced_wall.vert",
        "shaders/instanced_wall.frag",
        "instanced_wall_shader",
        ecs,
    );
    load_shader_into_library(
        "shaders/shadow.vert",
        "shaders/shadow.frag",
        "shadow_shader",
        ecs,
    );

    // indirect draw test
    let indirect_test = load_shader_into_library(
        "shaders/instanced_wall_arch.vert",
        "shaders/instanced_wall.frag",
        "indirect_instance_test",
        ecs,
    );
    ecs.spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: _brick,
            shader: indirect_test,
            transform: Transform::identity(),
        })
        .insert(IndirectDraw);

    // Create the starting scene
    ecs.spawn().insert_bundle(DrawableMeshBundle {
        mesh: floor,
        shader: vert_color,
        transform: Transform::identity(),
    });

    ecs.spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: road_pebbles,
            shader: road_shader,
            transform: Transform::identity(),
        })
        .insert(RoadComponent);

    /*
    ecs.spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: plane,
            shader: test,
            transform: Transform::from_translation(Vec3::new(0.0, 0.005, 0.0)),
        })
        .insert(DisplayTestMask);
        */

    // preview cube
    ecs.spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: cube,
            shader: vert_color,
            transform: Transform::identity(),
        })
        .insert(MousePreviewCube);

    log::info!("Finished startup");
}

fn load_mesh(path: &str) -> Mesh {
    let mesh_buffer = crate::utils::load_gltf::load_gltf_as_mesh_buffer(path);

    let mut mesh = Mesh::new();

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, mesh_buffer.positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_buffer.normals);

    mesh.set_indices(mesh_buffer.indices);

    if mesh_buffer.colors.is_empty() {
        mesh.add_color();
        //mesh.set_attribute(
        //    Mesh::ATTRIBUTE_COLOR,
        //    vec![[1.0, 1.0, 1.0]; mesh_buffer.positions.len()],
        //);
    } else {
        mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, mesh_buffer.colors);
    }

    mesh
}

fn load_mesh_into_library(mesh: Mesh, name: &str, ecs: &mut World) -> Handle<Mesh> {
    res_mut::<AssetMeshLibrary>(ecs).add(Asset::new(mesh).name(name))
}

fn load_shader_into_library(
    vertex_shader_path: &str,
    fragment_shader_path: &str,
    name: &str,
    ecs: &mut World,
) -> Handle<ShaderProgram> {
    let shader_program = ShaderProgram::new(vertex_shader_path, fragment_shader_path).unwrap();
    res_mut::<ShaderWatch>(ecs).watch(&shader_program);
    res_mut::<AssetShaderLibrary>(ecs).add(Asset::new(shader_program).name(name))
}
