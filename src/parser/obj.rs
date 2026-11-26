use std::{fs::File};
use std::io::{BufRead, BufReader};
use crate::utils::str_arr_to_f32;

pub type ObjCorner = (usize, usize, usize);
pub type ObjFace = Vec<ObjCorner>;


#[derive(Debug)]
pub struct ObjGroup {
    name: String,
    pub material_name: Option<String>,
    pub material_id: Option<usize>,
    pub faces: Vec<ObjFace>, 
}
impl ObjGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            material_name: None,
            material_id: None,
            faces: Vec::new(),
        }
    }
}
#[derive(Debug)]
pub struct  ObjObject {
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
pub struct ObjBuilder {
    pub(crate) position: Vec<[f32; 3]>,
    pub(crate) normal: Vec<[f32;3]>,
    pub(crate) tex_coords: Vec<[f32;2]>,
    pub material_file: Option<String>,
    pub objects: Vec<ObjObject>, 
}

impl ObjBuilder {
    pub fn new() -> Self {
        Self {
            position: Vec::new(),
            normal: Vec::new(),
            tex_coords: Vec::new(),
            material_file: None,
            objects: Vec::new(),
        }
    }
    pub fn parse_file(mut self, file: File) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = BufReader::new(file);


        let mut group_index: usize = 0;
        let mut material_name: Option<String> = None;

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
                        .ok_or("Not enough position elements")?
                        .try_into()?;

                    self.position.push(arr);
                }

                "vt" => {
                    let arr = str_arr_to_f32(&parts[1..]);
                    let arr: [f32; 2] = arr.get(..2)
                        .ok_or("Not enough texture elements")?
                        .try_into()?;

                    self.tex_coords.push(arr);
                }

                "vn" => {
                    let arr = str_arr_to_f32(&parts[1..]);
                    let arr: [f32; 3] = arr.get(..3)
                        .ok_or("Not enough normal elements")?
                        .try_into()?;

                    self.normal.push(arr);
                }

                "f" => {
                    let v_count = self.position.len();
                    let vt_count = self.tex_coords.len();
                    let vn_count = self.normal.len();

                    let face = parts[1..].join(" ");

                    match self.parse_face(&face, v_count, vt_count, vn_count) {
                        Ok(vertices) => {
                            if current_object.is_none() {
                                current_object = Some(ObjObject::new("Default Object"));
                            }

                            if current_group.is_empty() {
                                current_group.push(ObjGroup::new("Default Group"));
                            }

                            if let Some(group) = current_group.get_mut(group_index) {
                                group.faces.push(vertices);
                                if let Some(name) = &material_name{
                                    group.material_name = Some(name.clone());
                                } 
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
                }

                "o" => {
                    let name = parts.get(1).unwrap_or(&"Default Object");

                    if let Some(mut object) = current_object.take() {
                        object.groups = std::mem::take(&mut current_group);
                        self.objects.push(object);
                        group_index = 0;
                    }

                    current_object = Some(ObjObject::new(*name));
                }

                "g" => {
                    let name = parts.get(1).unwrap_or(&"Default Group");

                    current_group.push(ObjGroup::new(*name));
                    group_index = current_group.len() - 1;
                }
                "mtllib" => {
                    let file = *parts.get(1).unwrap();
                    self.material_file = Some(file.into());
                },
                "usemtl" => {
                    let name = *parts.get(1).unwrap_or(&"Default Material");
                    if let Some(group) = current_group.get_mut(group_index) {
                        group.material_name = Some(name.into());
                        material_name = None;
                    }
                    else {
                        material_name = Some(name.into());
                    }
                }
                _ => {}
            }
        }

        // Flush the last object
        if let Some(mut object) = current_object.take() {
            object.groups = std::mem::take(&mut current_group);
            self.objects.push(object);
        }

        Ok(self)
    }

    fn resolve_index(&self, index: i64, count: usize) -> Result<usize, String> {
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
        &self,
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
                    self.resolve_index(v, v_count)? - 1,
                    if vt == 0 { 0 } else { self.resolve_index(vt, vt_count)? - 1 },
                    if vn == 0 { 0 } else { self.resolve_index(vn, vn_count)? - 1 },
                ))
            })
            .collect()
    }

}