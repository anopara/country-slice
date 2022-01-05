use bevy_app::EventReader;
use bevy_ecs::prelude::*;
use glam::{Mat4, Vec2, Vec3};

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, Handle},
    components::{Transform, UiPrompt, UiPromptDebugPreview},
    render::{
        camera::{Camera, MainCamera},
        mesh::Mesh,
    },
    window_events::{CursorMoved, WindowSize},
};

pub fn ui_prompts(
    query: Query<(&UiPrompt, &Transform, &Handle<Mesh>)>,
    query_debug: Query<(&UiPromptDebugPreview, &Handle<Mesh>)>,
    mut cursor: EventReader<CursorMoved>,
    main_camera: Res<MainCamera>,
    window_size: Res<WindowSize>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
) {
    // for every vertex of the cube, transform it into screenspace
    let mut mesh_pos_ws = get_mesh_vertex_positions(*query.single().unwrap().2, &mut assets_mesh);
    mesh_pos_ws = mesh_pos_ws
        .iter()
        .map(|p| {
            query
                .single()
                .unwrap()
                .1
                .compute_matrix()
                .transform_point3(*p)
        })
        .collect();

    let mesh_pos_ss: Vec<_> = mesh_pos_ws
        .iter()
        .map(|p| from_ws_to_screenspace(*p, &window_size, &main_camera.camera))
        .collect();
    //dbg!(mesh_pos_ws);
    //dbg!(mesh_pos_ss.clone());

    // find the bounding box of them in screenspace -> TODO: check how this looks, does it look correct????
    let bbx_ss = bbx_screenspace(&mesh_pos_ss);
    let debug_ss_pos = vec![
        bbx_ss.0,
        Vec2::new(bbx_ss.0.x, bbx_ss.1.y),
        bbx_ss.1,
        Vec2::new(bbx_ss.1.x, bbx_ss.0.y),
        bbx_ss.0,
    ];
    let debug_ws_pos: Vec<_> = debug_ss_pos
        .iter()
        .map(|p| from_screenspace_to_ws(*p, window_size.into_vec2(), &main_camera.camera))
        .collect();
    update_debug_mesh(
        debug_ws_pos,
        query_debug.single().unwrap().1,
        &mut assets_mesh,
    );

    //dbg!(bbx_ss);

    if let Some(cursor_latest) = cursor.iter().last() {
        dbg!(cursor_latest.pos);
        let bbx_min_x = bbx_ss.0.x;
        let bbx_max_x = bbx_ss.1.x;

        let bbx_min_y = bbx_ss.0.y;
        let bbx_max_y = bbx_ss.1.y;

        if cursor_latest.pos.x > bbx_min_x
            && cursor_latest.pos.x < bbx_max_x
            && (cursor_latest.pos.y) > bbx_min_y
            && (cursor_latest.pos.y) < bbx_max_y
        {
            println!("Inside~ {} vs {:?}", cursor_latest.pos, bbx_ss);
        } else {
            println!("NOPE~ {} vs {:?}", cursor_latest.pos, bbx_ss);
        }
    }

    // expand that box by X pixels padding
    // check if mouse is inside that 2d volume
    // print something to console if so
    // TODO: also start looking into events!
}

pub fn from_ws_to_screenspace(ws_pos: Vec3, window_size: &WindowSize, camera: &Camera) -> Vec2 {
    let gl_pos = camera.perspective_projection * camera.world_to_camera_view() * ws_pos.extend(1.0);

    let mut ndc = gl_pos.truncate() / gl_pos.w;
    ndc.y = -ndc.y;

    (ndc.truncate() + 1.0) / 2.0 * window_size.into_vec2()
}

pub fn from_screenspace_to_ws(pos_ss: Vec2, screen_size: Vec2, camera: &Camera) -> Vec3 {
    let mut ndc = (Vec2::new(pos_ss.x, pos_ss.y) / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    ndc.y = -ndc.y;

    let ndc_to_world: Mat4 = camera.transform * camera.perspective_projection.inverse();
    let pos_ws = ndc_to_world.project_point3(ndc.extend(0.0));

    pos_ws
}

pub fn get_mesh_vertex_positions(
    handle: Handle<Mesh>,
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
) -> Vec<Vec3> {
    let mesh = assets_mesh.get_mut(handle).unwrap();
    let mesh_ws_pos = mesh.attributes.get(Mesh::ATTRIBUTE_POSITION).unwrap();

    if let crate::render::mesh::VertexAttributeValues::Float32x3(positions) = mesh_ws_pos {
        positions.iter().map(|p| Vec3::from_slice(p)).collect()
    } else {
        panic!()
    }
}

// TODO: ask Tom pub fn bbx_screenspace<'a>(ss_pos: impl Iterator<Item = &'a Vec2>) -> (Vec2, Vec2) {
pub fn bbx_screenspace(ss_pos: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::ZERO;

    for p in ss_pos {
        // find min
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        // find max
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }

    (min, max)
}

fn update_debug_mesh(
    positions_ws: Vec<Vec3>,
    mesh_handle: &Handle<Mesh>,
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
) {
    let mesh = assets_mesh.get_mut(*mesh_handle).expect("MEOW####");
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions_ws
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.set_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[1.0, 0.0, 0.0]; positions_ws.len()],
    );
    mesh.set_indices((0..positions_ws.len()).map(|i| i as u32).collect());
}
