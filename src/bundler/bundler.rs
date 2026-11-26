use std::collections::HashMap;

use crate::parser::{MaterialRecords, MtlMaterial, ObjBuilder, ObjFace, ObjObject};

#[derive(Debug)]
pub struct ObjMesh {
    pub position: Vec<[f32; 3]>,
    pub normal: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub objects: Vec<ObjObject>,
    pub materials: Vec<MtlMaterial>,
}

#[derive(Debug)]
pub struct ObjGroup {
    pub name: String,
    pub material_id: Option<usize>,
    pub faces: Vec<ObjFace>,
}

pub fn bundle(
    mut builder: ObjBuilder,
    material_records: MaterialRecords,
) -> ObjMesh {

    // Convert HashMap<String, MtlMaterial> into Vec<MtlMaterial>
    let mut materials = Vec::new();

    // Material name -> material index
    let mut material_map: HashMap<String, usize> = HashMap::new();

    for (name, material) in material_records.materials {
        let material_id = materials.len();

        material_map.insert(name, material_id);
        materials.push(material);
    }

    // Resolve material names into material indices
    for object in &mut builder.objects {
        for group in &mut object.groups {
            group.material_id = group
                .material_name
                .as_ref()
                .and_then(|name| material_map.get(name).copied());
        }
    }

    // Build final mesh
    ObjMesh {
        position: builder.position,
        normal: builder.normal,
        tex_coords: builder.tex_coords,
        objects: builder.objects,
        materials,
    }
}