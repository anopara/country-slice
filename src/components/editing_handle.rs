pub enum HandleLocation {
    StartOfCurve,
    EndOfCurve,
}

//#[derive(Bundle)]
pub struct EditingHandle {
    pub parent_curve: usize, // which curve does it belong
    pub location: HandleLocation,
    //pub trigger_area: TriggerArea,
    //pub trigger_area_preview: TriggerAreaPreview,
}

impl EditingHandle {
    pub fn new(parent_curve: usize, handle_type: HandleLocation) -> Self {
        Self {
            parent_curve,
            location: handle_type, //trigger_area: TriggerArea::new(20, Transform::from_translation(position)),
        }
    }

    /*
    pub fn add_world_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        self.trigger_area
            .add_world_space_preview(assets_mesh, assets_shader, commands)
    }

    pub fn add_screen_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        self.trigger_area
            .add_screen_space_preview(assets_mesh, assets_shader, commands)
    }
    */
}
