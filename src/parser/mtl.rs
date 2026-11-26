use std::collections::HashMap;
use std::{fs::File};
use std::io::{BufRead, BufReader};

use crate::utils::str_arr_to_f32;

#[derive(Debug)]
pub struct MtlMaterial {
    pub name: String,

    // Colors
    pub ambient: Option<[f32; 3]>,
    pub diffuse: Option<[f32; 3]>,
    pub specular: Option<[f32; 3]>,
    pub emissive: Option<[f32; 3]>,
    pub transmission: Option<[f32; 3]>,

    // Lighting
    pub shininess: Option<f32>,
    pub optical_density: Option<f32>,
    pub dissolve: Option<f32>,
    pub transparency: Option<f32>,
    pub illumination: Option<u32>,

    // Texture maps
    pub ambient_map: Option<String>,
    pub diffuse_map: Option<String>,
    pub specular_map: Option<String>,
    pub emissive_map: Option<String>,
    pub shininess_map: Option<String>,
    pub opacity_map: Option<String>,
    pub bump_map: Option<String>,
    pub displacement_map: Option<String>,
    pub decal_map: Option<String>,
    pub reflection_map: Option<String>,
}
impl MtlMaterial {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),

            // Colors
            ambient: None,
            diffuse: None,
            specular: None,
            emissive: None,
            transmission: None,

            // Lighting
            shininess: None,
            optical_density: None,
            dissolve: None,
            transparency: None,
            illumination: None,

            // Texture maps
            ambient_map: None,
            diffuse_map: None,
            specular_map: None,
            emissive_map: None,
            shininess_map: None,
            opacity_map: None,
            bump_map: None,
            displacement_map: None,
            decal_map: None,
            reflection_map: None,
        }
    }
}
#[derive(Debug)]
pub struct MaterialRecords {
    pub materials: HashMap<String,  MtlMaterial>
}
impl MaterialRecords {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new()
        }
    }
    fn set_material_color(
        &self,
        material: &mut Option<MtlMaterial>,
        values: &[&str],
        property: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(material) = material.as_mut() else {
            return Ok(());
        };

        let arr = str_arr_to_f32(values);

        let color: [f32; 3] = arr.get(..3)
            .ok_or("Not enough color elements")?
            .try_into()?;

        match property {
            "Ka" => material.ambient = Some(color),
            "Kd" => material.diffuse = Some(color),
            "Ks" => material.specular = Some(color),
            "Ke" => material.emissive = Some(color),
            "Tf" => material.transmission = Some(color),
            _ => {}
        }

        Ok(())
    }
    fn set_lighting_value(
        &self, 
        material: &mut Option<MtlMaterial>,
        value: &str,
        property: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(material) = material.as_mut() else {
            return Ok(());
        };
        let value = value.parse::<f32>().expect("String not a f32 value");
        match property {
            "Ns" => material.shininess = Some(value),
            "Ni" => material.optical_density = Some(value),
            "d" => material.dissolve = Some(value),
            "Tr" => material.transparency = Some(value),
            _ => {}
        }

        Ok(())
    }
    fn set_material_map(
        &self,
        material: &mut Option<MtlMaterial>,
        value: &str,
        property: &str,
    ) {
        let Some(material) = material.as_mut() else {
            return;
        };

        match property {
            "map_Ka" => material.ambient_map = Some(value.to_string()),
            "map_Kd" => material.diffuse_map = Some(value.to_string()),
            "map_Ks" => material.specular_map = Some(value.to_string()),
            "map_Ke" => material.emissive_map = Some(value.to_string()),
            "map_Ns" => material.shininess_map = Some(value.to_string()),
            "map_d" => material.opacity_map = Some(value.to_string()),
            "bump" | "map_bump" => material.bump_map = Some(value.to_string()),
            "disp" => material.displacement_map = Some(value.to_string()),
            "decal" => material.decal_map = Some(value.to_string()),
            "refl" => material.reflection_map = Some(value.to_string()),
            _ => {}
        }
    }
    pub fn parse_file(mut self, file: File) ->  Result<Self, Box<dyn std::error::Error>> {
        let reader = BufReader::new(file);
        let mut current_mat: Option<MtlMaterial> = None;
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "newmtl" => {
                    let name = parts[1];
                    if let Some(material) = current_mat.take() {
                        self.materials.insert(material.name.clone(), material);
                        current_mat = Some(MtlMaterial::new(name))
                    }
                    else {
                        current_mat = Some(MtlMaterial::new(name));
                    }
                },
                "Ka" | "Kd" | "Ks" | "Ke" | "Tf" => {
                    self.set_material_color(
                        &mut current_mat,
                        &parts[1..],
                        parts[0],
                    )?;
                },
                "Ns" | "Ni" | "d" | "Tr" => {
                    self.set_lighting_value(&mut current_mat, parts[1], parts[0])?;
                },
                "illum" => {
                    if let Some(material) = current_mat.as_mut() {
                        material.illumination = Some(parts[1].parse()?);
                    }
                },
                "map_Ka" | "map_Kd" | "map_Ks" | "map_Ke" |
                "map_Ns" | "map_d" | "bump" | "map_bump" |
                "disp" | "decal" | "refl" => {
                    self.set_material_map(
                        &mut current_mat,
                        parts[1],
                        parts[0],
                    );
                }

                _ => {}
            }
        }
        if let Some(material) = current_mat.take() {
            self.materials.insert(material.name.clone(), material);
        }

        Ok(self)
    }
}