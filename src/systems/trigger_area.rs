use bevy_app::EventReader;
use bevy_ecs::prelude::*;
use glam::{Mat4, Vec2, Vec3};

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, Handle},
    components::{Transform, TriggerArea},
    render::{
        camera::{Camera, MainCamera},
        mesh::Mesh,
    },
    resources::LastHoveredTriggerArea,
    window_events::{CursorMoved, WindowSize},
};

//TODO: next -> walls should have ui prompts on their ends
// if start drawing from the end, continue that curve
// REDO THE RENDERING LOOP, make a screen space rendering pass

pub fn trigger_area(
    mut last_hovered: ResMut<LastHoveredTriggerArea>,

    mut q1: Query<(Entity, &mut TriggerArea)>,
    mut q2: Query<&mut Handle<Mesh>>,
    mut cursor: EventReader<CursorMoved>,
    main_camera: Res<MainCamera>,
    window_size: Res<WindowSize>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
) {
    let cursor_latest_position;
    if let Some(c) = cursor.iter().last() {
        cursor_latest_position = c.pos;
    } else {
        return;
    }

    let mut prompt_preview = Vec::new(); // borrow checker workaround

    for (entity, mut trigger_area) in q1.iter_mut() {
        let trigger_area_ss = trigger_area
            .iter_ws_bounds()
            .map(|p| from_ws_to_screenspace(*p, &window_size, &main_camera.camera));

        let mut bbx = bbx_screenspace(trigger_area_ss);
        bbx.add_padding(trigger_area.padding);

        if cursor_latest_position.x > bbx.min.x
            && cursor_latest_position.x < bbx.max.x
            && (cursor_latest_position.y) > bbx.min.y
            && (cursor_latest_position.y) < bbx.max.y
        {
            trigger_area.is_mouse_over = true;
            // TODO: this will not sort if two areas overlap, and will just send an event for both!
            last_hovered.0 = Some(entity);
        } else {
            trigger_area.is_mouse_over = false;
            last_hovered.0 = None;
        }

        if let Some(debug_preview) = trigger_area.ss_preview {
            prompt_preview.push((debug_preview, bbx));
        }
    }

    // Update Ui Debug Previews
    for (entity, bbx) in prompt_preview {
        update_debug_mesh(
            &q2.get_mut(entity).unwrap(),
            &bbx,
            &window_size,
            &main_camera.camera,
            &mut assets_mesh,
        );
    }
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

pub struct ScreenSpaceBoundingBox {
    min: Vec2,
    max: Vec2,
}

impl ScreenSpaceBoundingBox {
    pub fn add_padding(&mut self, v: usize) {
        self.min -= Vec2::new(v as f32, v as f32);
        self.max += Vec2::new(v as f32, v as f32);
    }
}

pub fn bbx_screenspace<'a>(ss_pos: impl Iterator<Item = Vec2>) -> ScreenSpaceBoundingBox {
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

    ScreenSpaceBoundingBox { min, max }
}

fn update_debug_mesh(
    mesh_handle: &Handle<Mesh>,
    bbx: &ScreenSpaceBoundingBox,
    window_size: &WindowSize,
    camera: &Camera,
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
) {
    let debug_ss_pos = vec![
        bbx.min,
        Vec2::new(bbx.min.x, bbx.max.y),
        bbx.max,
        Vec2::new(bbx.max.x, bbx.min.y),
        bbx.min,
    ];
    let positions_ws: Vec<_> = debug_ss_pos
        .iter()
        .map(|p| from_screenspace_to_ws(*p, window_size.into_vec2(), camera))
        .collect();

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
