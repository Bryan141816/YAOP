use std::collections::HashMap;
use std::{fs::File};
use std::io::{BufRead, BufReader};


type ObjCorner = (usize, usize, usize);
type ObjFace = Vec<ObjCorner>;

#[derive(Debug)]
struct ObjGroup {
    name: String,
    faces: Vec<ObjFace>, 
}
impl ObjGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            faces: Vec::new(),
        }
    }
}
#[derive(Debug)]
struct  ObjObject {
    pub name: String,
    pub groups: Vec<ObjGroup>
}
impl ObjObject {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            groups: Vec::new()
        }
    }
}

#[derive(Debug)]
struct ObjBuilder {
    pub(crate) position: Vec<[f32; 3]>,
    pub(crate) normal: Vec<[f32;3]>,
    pub(crate) text_coords: Vec<[f32;2]>,
    pub(crate) objects: Vec<ObjObject>, 
}
#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normal: [f32; 3],
}
#[derive(Debug)]
struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<usize>
}
fn str_arr_to_f32(str_arr: &[&str]) -> Vec<f32> {
    let mut arr = Vec::new();
    for str in str_arr {
        let num = str.parse::<f32>().expect("Failed to parse string as f32");
        arr.push(num);
    }
    arr
} 
fn resolve_index(index: i64, count: usize) -> Result<usize, String> {
    if index == 0 {
        return Err("OBJ index cannot be zero".to_string());
    }

    let resolved = if index < 0 {
        count as i64 + index + 1
    } else {
        index
    };

    if resolved <= 0 || resolved > count as i64 {
        return Err("Index out of bounds".to_string());
    }

    Ok(resolved as usize)
}

pub fn parse_face(
    face: &str,
    v_count: usize,
    vt_count: usize,
    vn_count: usize,
) -> Result<Vec<ObjCorner>, String> {
    face.split_whitespace()
        .map(|vertex| {
            let mut parts = vertex.split('/');

            let v = parts.next()
                .ok_or("Missing position index")?
                .parse::<i64>()
                .map_err(|_| "Invalid position index")?;

            let vt = parts.next()
                .filter(|s| !s.is_empty())
                .unwrap_or("0")
                .parse::<i64>()
                .map_err(|_| "Invalid texture index")?;

            let vn = parts.next()
                .filter(|s| !s.is_empty())
                .unwrap_or("0")
                .parse::<i64>()
                .map_err(|_| "Invalid normal index")?;

            Ok((
                resolve_index(v, v_count)? - 1,
                if vt == 0 { 0 } else { resolve_index(vt, vt_count)? - 1 },
                if vn == 0 { 0 } else { resolve_index(vn, vn_count)? - 1 },
            ))
        })
        .collect()
}
fn build_mesh_from_obj(positions: Vec<[f32;3]>, tex_coords: Vec<[f32;2]>, normals: Vec<[f32;3]>, faces: Vec<ObjFace>) -> Mesh {
    let mut vertex_map:HashMap<ObjCorner, usize> = HashMap::new();
    let mut indices = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();

    for face in faces {
        for i in 1..face.len() - 1 {
            for corner in [face[0], face[i], face[i + 1]] {
                 let (v, vt, vn) = corner;
                let index = *vertex_map.entry(corner).or_insert_with(|| {
                    let index = vertices.len();

                    let new_vertex = Vertex {
                        position: positions[v],
                        tex_coord:  tex_coords.get(vt).copied().unwrap_or([0.0, 0.0]),
                        normal: normals.get(vn).copied().unwrap_or([0.0, 0.0, 0.0]),
                    };

                    vertices.push(new_vertex);

                    index
                });

                indices.push(index);
            }
        }
    }
    Mesh { vertices, indices }
}
pub fn parse_obj(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut bulider = ObjBuilder {
        position: Vec::new(),
        normal: Vec::new(),
        text_coords: Vec::new(),
        objects: Vec::new(),
    };
    let mut group_index: usize = 0;

    let mut current_object: Option<ObjObject> = None;
    let mut current_group: Vec<ObjGroup> = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                let arr = str_arr_to_f32(&parts[1..]);
                let arr: [f32; 3] = arr.get(..3)
                    .expect("Not enough position elements")
                    .try_into()
                    .expect("Invalid array length");

                bulider.position.push(arr);
            },
            "vt" => {
                let arr = str_arr_to_f32(&parts[1..]);
                let arr: [f32; 2] = arr.get(..2)
                    .expect("Not enough texture elements")
                    .try_into()
                    .expect("Invalid array length");

                bulider.text_coords.push(arr);
            },
            "vn" => {
                let arr = str_arr_to_f32(&parts[1..]);
                let arr: [f32; 3] = arr.get(..3)
                    .expect("Not enough normal elements")
                    .try_into()
                    .expect("Invalid array length");

                bulider.normal.push(arr);
            },
            "f" => {
                let v_count = bulider.position.len();
                let vt_count = bulider.text_coords.len();
                let vn_count = bulider.normal.len();

                let face = parts[1..].join(" ");

                match parse_face(&face, v_count, vt_count, vn_count) {
                    Ok(vertices) => {
                        if current_object.is_none() {
                            current_object = Some(ObjObject::new("Default Object"));
                        }
                        if current_group.is_empty() {
                            let group = ObjGroup::new("Default Group");
                            current_group.push(group);
                        }
                        if let Some(mut group) = current_group.get_mut(group_index){
                            group.faces.push(vertices);
                        }
                    }
                    Err(err) => {
                        println!(
                            "Face Error at line {}: {}",
                            line_number + 1,
                            err
                        );
                    }
                }
            },
            "o" => {
                let name = parts[1];
                if current_object.is_none() {
                    current_object = Some(ObjObject::new(name));
                }
                else {
                    if let Some(mut object) = current_object {
                        if !current_group.is_empty() {
                            object.groups = std::mem::take(&mut current_group);
                            bulider.objects.push(object);
                            current_group = Vec::new();
                            group_index = 0;
                        }
        
                    }
                    current_object = Some(ObjObject::new(name));
                }
            },
            "g" => {
                let name = parts[1];
                if !current_group.is_empty() {
                    group_index += 1;
                }
                let group = ObjGroup::new(name);
                current_group.push(group);
            },
            // "usemtl" => println!("Material: {:?}", &parts[1..]),
            // "#" => println!("Comment: {:?}", &parts[1..]),
            _ => println!("Unknown at line {}", line_number + 1),
        }
    }
    if !current_object.is_none() && !current_group.is_empty() {
        if let Some(mut object) = current_object.take(){
            object.groups = current_group;
            bulider.objects.push(object);
        }
    }
    println!("{:#?}", bulider.objects);
    // let mesh = build_mesh_from_obj(bulider.position, bulider.text_coords, bulider.normal, bulider.faces);

    // println!("{:#?}", mesh);
    Ok(())
}