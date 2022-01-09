use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Asset},
    render::mesh::Mesh,
};
use bevy_ecs::prelude::*;
use glam::Vec3;

use super::{DrawableMeshBundle, Transform};

// Screenspace Bounding box check
#[derive(Clone)]
pub struct TriggerArea {
    pub is_mouse_over: bool,
    pub padding: usize, // padding of the interaction bounding box in screen-space in pixels
    pub size: Vec3,
    pub transform: Transform,

    pub ws_preview: Option<Entity>,
    pub ss_preview: Option<Entity>, //TODO: make it a component that renders in a special way

    cache_ws_bounds: Option<Vec<Vec3>>, // cache
}

impl TriggerArea {
    pub fn new(padding: usize, transform: Transform) -> Self {
        Self {
            is_mouse_over: false,
            padding,
            size: Vec3::ONE * 0.1,
            transform,
            ws_preview: None,
            ss_preview: None,
            cache_ws_bounds: None,
        }
    }

    pub fn update_transform(&mut self, position: Vec3) {
        self.transform.translation = position;
        self.cache_ws_bounds = None;
    }

    pub fn iter_ws_bounds<'a>(&'a mut self) -> impl Iterator<Item = &Vec3> + 'a {
        if self.cache_ws_bounds.is_none() {
            self.cache_ws_bounds = Some(
                Vec::<Vec3>::from(crate::geometry::cube::Box::new(
                    self.size.x,
                    self.size.y,
                    self.size.z,
                ))
                .iter()
                .map(|ls| self.transform.compute_matrix().transform_point3(*ls))
                .collect(),
            )
        }

        self.cache_ws_bounds.as_ref().unwrap().iter()
    }

    #[allow(dead_code)]
    pub fn add_world_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        let shader = assets_shader
            .get_handle_by_name("vertex_color_shader")
            .unwrap();

        let entity = commands
            .spawn()
            .insert_bundle(DrawableMeshBundle {
                mesh: assets_mesh.add(self.mesh_asset()),
                shader,
                transform: self.transform.clone(),
            })
            .id();

        self.ws_preview = Some(entity);

        entity
    }

    pub fn add_screen_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        let shader = assets_shader
            .get_handle_by_name("vertex_color_shader")
            .unwrap();
        let entity = commands
            .spawn()
            .insert_bundle(DrawableMeshBundle {
                mesh: assets_mesh.add(TriggerAreaPreview::mesh_asset()),
                shader,
                transform: Transform::identity(),
            })
            .insert(TriggerAreaPreview)
            .insert(crate::components::GLDrawMode(gl::LINE_STRIP))
            .id();

        self.ss_preview = Some(entity);

        entity
    }

    pub fn mesh_asset(&self) -> Asset<Mesh> {
        let mut mesh = Mesh::from(crate::geometry::cube::Box::new(
            self.size.x,
            self.size.y,
            self.size.z,
        ));
        mesh.add_color([0.0, 0.7, 1.0]);
        Asset::new(mesh).name("trigger_area")
    }
}

pub struct TriggerAreaPreview; //2D line strip to draw to show the bounding box in screenspace

impl TriggerAreaPreview {
    pub fn mesh_asset() -> Asset<Mesh> {
        // TODO: make this into a component? and has a custom draw for it in the render loop?
        let mut _preview = Mesh::new();
        _preview.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
        );
        _preview.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 0.0, 0.0]; 4]);
        _preview.set_indices(vec![0, 1, 2, 3]);

        Asset::new(_preview).name("trigger_area_preview")
    }
}
