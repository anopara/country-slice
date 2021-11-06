use bevy::prelude::*;
//use genmesh::{Quad, Triangle, Vertex};
//use genmesh::{Triangulate, Vertices};

pub struct Grid {
    //cells: Vec<f32>,
    dims: (usize, usize),
    origin: Vec2,
    size: Vec2,
}

impl Grid {
    pub fn new(origin: Vec2, size: Vec2, dims: (usize, usize)) -> Self {
        Self {
            origin,
            size,
            //cells: vec![0.0; dims.0 * dims.1],
            dims,
        }
    }

    // Could also just make a single quad and draw grid as a shader?
    pub fn create_debug_mesh(
        &self,
        mesh_assets: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        commands: &mut Commands,
    ) {
        let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

        let bbx_min = self.origin - self.size / 2.0;
        let cell_size = Vec2::new(
            self.size.x / (self.dims.0 as f32),
            self.size.y / (self.dims.1 as f32),
        );

        println!("bbx_min: {:?}", bbx_min);
        println!("cell_size: {:?}", cell_size);

        let (positions, uvs): (Vec<[f32; 3]>, Vec<[f32; 2]>) = (0..self.dims.0)
            .map(|row| {
                (0..self.dims.1)
                    .map(|col| {
                        let p = bbx_min + cell_size * Vec2::new(col as f32, row as f32);
                        let uv = [
                            (col as f32) / (self.dims.0 as f32 - 1.0),
                            (row as f32) / (self.dims.1 as f32 - 1.0),
                        ];
                        ([p.x, 0.05, p.y], uv)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .unzip();

        let indices: Vec<u32> = (0..self.dims.0)
            .map(|row| {
                (0..self.dims.1)
                    .filter_map(|col| {
                        let next_row = row + 1;
                        let next_col = col + 1;
                        if next_row < self.dims.1 && next_col < self.dims.0 {
                            let current_index = row * self.dims.0 + col;
                            Some(vec![
                                // triangle 1
                                current_index as u32,
                                (current_index + self.dims.0) as u32,
                                (current_index + 1) as u32,
                                // triangle 2
                                (current_index + 1) as u32,
                                (current_index + self.dims.0) as u32,
                                (current_index + self.dims.0 + 1) as u32,
                            ])
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        for p in &positions {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh_assets.add(Mesh::from(shape::Cube { size: 0.05 })),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(p[0], p[1], p[2]),
                //render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                //    self.debug_mesh_pipeline.clone(),
                //)]),
                ..Default::default()
            });
        }

        mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 1.0, 0.0]; positions.len()],
        );
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        let handle = mesh_assets.add(mesh);

        commands.spawn_bundle(PbrBundle {
            mesh: handle,
            material: materials.add(Color::rgb(0.0, 1.0, 1.0).into()),
            //render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            //    self.debug_mesh_pipeline.clone(),
            //)]),
            ..Default::default()
        });

        /*
        let plane =
            genmesh::generators::Plane::subdivide(self.subdivision.0 + 1, self.subdivision.1 + 1);

        let positions: Vec<[f32; 3]> = plane
            .triangulate()
            .vertices()
            .map(|v| [v.pos.x, v.pos.y + 0.05, v.pos.z]) // put it slightly above
            .collect();

        let uvs: Vec<[f32; 2]> = plane
            .map(|_quad| vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]])
            .flatten()
            .collect();

        mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 1.0, 0.0]; positions.len()],
        );
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
            (0..(positions.len() as u32)).collect(),
        )));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        let handle = mesh_assets.add(mesh);

        commands.spawn_bundle(PbrBundle {
            mesh: handle,
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            //render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            //    self.debug_mesh_pipeline.clone(),
            //)]),
            ..Default::default()
        });
        */
    }
}
