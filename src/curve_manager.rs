use bevy::prelude::*;

pub struct CurveManager {
    // points placed by the user
    pub point_positions: Vec<Vec3>,
    pub preview_mesh_handle: Option<Handle<Mesh>>,
    pub brick_mesh_handle: Option<Handle<Mesh>>,
}

impl CurveManager {
    pub fn new() -> Self {
        Self {
            point_positions: Vec::new(),
            preview_mesh_handle: None,
            brick_mesh_handle: None,
        }
    }

    pub fn smooth_positions(&self) -> Vec<Vec3> {
        let points_per_segment = 10;
        let smoothing_steps = 50;

        // resample curve
        let mut resampled: Vec<Vec3> = Vec::new();
        for (i, current_pos) in self.point_positions.iter().enumerate() {
            if let Some(next_pos) = self.point_positions.get(i + 1) {
                let dir = *next_pos - *current_pos;
                resampled.extend(
                    &(0..points_per_segment)
                        .map(|s| *current_pos + dir * (s as f32 / points_per_segment as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                // if last point, just add
                resampled.push(*current_pos);
            }
        }

        // smooth
        let mut total_smoothed = resampled.clone();
        for _ in 0..smoothing_steps {
            let mut current_iter_smooth = total_smoothed.clone();
            for (i, current_pos) in total_smoothed.iter().enumerate() {
                if let (Some(prev_pos), Some(next_pos)) =
                    (total_smoothed.get(i - 1), total_smoothed.get(i + 1))
                {
                    let avg: Vec3 = (*prev_pos + *next_pos) / 2.0;
                    current_iter_smooth[i] = *current_pos + (avg - *current_pos) * 0.5;
                }
            }
            total_smoothed = current_iter_smooth;
        }

        total_smoothed
    }

    /*
    fn to_vertices(&self) -> Vec<[f32; 3]> {
        let mut one_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1] + 1.0, p[2]], [p[0], p[1], p[2]]])
            .flatten()
            .collect();
        let mut other_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1], p[2]], [p[0], p[1] + 1.0, p[2]]])
            .flatten()
            .collect();
        other_side.reverse();
        one_side.extend(&other_side);
        one_side
    }
    */

    fn to_trimesh(&self) -> tri_mesh::mesh::Mesh {
        let curve_positions = self.smooth_positions();
        println!(
            "Smoothed {} points to {}",
            self.point_positions.len(),
            curve_positions.len()
        );

        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<f64> = Vec::new();
        for quad_index in 0..(curve_positions.len() - 1) {
            let start_point = curve_positions[quad_index];
            let end_point = curve_positions[quad_index + 1];

            let vert_index_start = (positions.len() / 3) as u32;

            positions.extend(&vec![
                // start vertex
                start_point[0] as f64,
                start_point[1] as f64,
                start_point[2] as f64,
                // offset up
                start_point[0] as f64,
                start_point[1] as f64 + 1.0,
                start_point[2] as f64,
                // end vertex
                end_point[0] as f64,
                end_point[1] as f64,
                end_point[2] as f64,
                // offset up
                end_point[0] as f64,
                end_point[1] as f64 + 1.0,
                end_point[2] as f64,
            ]);

            indices.extend(
                &([0, 1, 2, 1, 3, 2]
                    .iter()
                    .map(|i| i + vert_index_start)
                    .collect::<Vec<_>>()),
            )
        }

        /*
        // Construct a mesh from indices and positions buffers.
        let mut indices: Vec<_> = (0..(self.point_positions.len() - 1) * 2)
            .map(|i| {
                let mut ind = vec![i as u32, (i + 1) as u32, (i + 2) as u32];
                if i % 2 != 0 {
                    ind.reverse()
                };
                ind
            })
            .flatten()
            .collect();
        let mut other_side = indices.clone();
        other_side.reverse();
        //indices.extend(&other_side);

        let positions = self
            .point_positions
            .iter()
            .map(|p| {
                vec![
                    // original vertex
                    p[0] as f64,
                    p[1] as f64,
                    p[2] as f64,
                    // offset up
                    p[0] as f64,
                    p[1] as f64 + 1.0,
                    p[2] as f64,
                ]
            })
            .flatten()
            .collect();
            */

        println!("-----Building mesh: ");
        println!("indices: {}", indices.len());
        //println!("positions: {:?}", positions);

        let mesh = tri_mesh::MeshBuilder::new()
            .with_indices(indices)
            .with_positions(positions)
            .build()
            .unwrap();

        println!("-----Done");

        mesh
    }

    pub fn populate_bevy_mesh(&self, bevy_mesh: &mut Mesh) {
        //let vert_pos = self.to_vertices();
        //let vert_count = vert_pos.len();
        let tri_mesh = self.to_trimesh();
        let vert_count = tri_mesh.vertex_iter().count();

        let positions: Vec<[f32; 3]> = tri_mesh
            .positions_buffer_f32()
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let normals: Vec<[f32; 3]> = tri_mesh
            .normals_buffer_f32()
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let mut indices = tri_mesh.indices_buffer();
        let mut other_side = indices.clone();
        other_side.reverse();
        indices.extend(&other_side);

        //println!("indices {:?}", indices);
        //println!("normals {:?}", normals);
        //println!("positions {:?}", positions);

        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals); //vec![[1.0, 0.0, 0.0]; vert_count]);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; vert_count]);
        bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
            indices, //(0..vert_count).map(|i| i as u32).collect(),
        )));
    }
}
