use crate::render::mesh::Mesh;

pub fn load_json_as_mesh(path: &str) -> Result<Mesh, String> {
    let data = std::fs::read_to_string(path).unwrap();

    // Parse the string of data into serde_json::Value.
    let v: serde_json::Value = serde_json::from_str(&data).unwrap();
    //.unwrap_or(Err(format!("Serde Json: couldn't parse {}", path)))?;

    // Create Mesh
    let mut mesh = Mesh::new();

    // Read indices
    let indices: Vec<u32> = v["indices"]["buffer"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    mesh.set_indices(indices);

    // Read attributes
    let attributes: Vec<_> = v["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    for attrib_name in attributes {
        dbg!(attrib_name);
        let strrr = format!("{}", v[attrib_name]["type"]);
        match strrr.as_ref() {
            r#"["float",4]"# => {
                let values: Vec<[f32; 4]> = v[attrib_name]["buffer"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        let d: Vec<_> = v
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap() as f32)
                            .collect();
                        [d[0], d[1], d[2], d[3]]
                    })
                    .collect();
                dbg!(values.len());
                mesh.set_attribute(attrib_name, values)
            }
            r#"["float",3]"# => {
                let values: Vec<[f32; 3]> = v[attrib_name]["buffer"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        let d: Vec<_> = v
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap() as f32)
                            .collect();
                        [d[0], d[1], d[2]]
                    })
                    .collect();
                dbg!(values.len());
                mesh.set_attribute(attrib_name, values)
            }
            r#"["int",1]"# => {
                let values: Vec<i32> = v[attrib_name]["buffer"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap() as i32)
                    .collect();
                dbg!(values.len());
                mesh.set_attribute(attrib_name, values)
            }
            _ => {
                return Err(format!(
                    "load_json_as_mesh: unrecognized attrib {} with type {}",
                    attrib_name, strrr
                ));
            }
        }
    }

    Ok(mesh)
}
