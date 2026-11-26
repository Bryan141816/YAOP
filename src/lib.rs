pub mod parser;
pub mod mesh;
pub mod utils;
pub mod bundler;
use std::path::Path;
use std::{fs::File};
use crate::bundler::{ObjMesh, bundle};

use crate::parser::{Builder, MaterialRecords, Model, ObjBuilder };



pub fn convert_to_mesh(obj: ObjMesh) -> Model {
    Builder::new().convert_to_mesh(obj)
}

pub fn parse_obj(path: &str) -> Result<ObjMesh, Box<dyn std::error::Error>> {
    let obj_path = Path::new(path);

    let file = File::open(obj_path)?;
    let builder = ObjBuilder::new().parse_file(file)?;

    let Some(material_file) = &builder.material_file else {
        return Ok(ObjMesh {
            position: builder.position,
            normal: builder.normal,
            tex_coords: builder.tex_coords,
            objects: builder.objects,
            materials: Vec::new(),
        })
        
    };

    let obj_directory = obj_path.parent().unwrap_or(Path::new("."));
    let mtl_path = obj_directory.join(material_file);

    let mat_file = File::open(mtl_path)?;

    let material_records = MaterialRecords::new()
        .parse_file(mat_file)?;

    Ok(bundle(builder, material_records))
}