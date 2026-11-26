use std::collections::HashMap;
use crate::bundler::ObjMesh;
use crate::{
    mesh::{Mesh, Vertex},
    parser::{ObjCorner, ObjFace, MtlMaterial},
};

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<MtlMaterial>,
}

impl Model {
    pub fn new(materials: Vec<MtlMaterial>) -> Self {
        Self {
            meshes: Vec::new(),
            materials,
        }
    }
}

pub struct Builder;

impl Builder {
    pub fn new() -> Self {
        Self
    }

    fn build_mesh_from_obj(
        &self,
        positions: &[[f32; 3]],
        tex_coords: &[[f32; 2]],
        normals: &[[f32; 3]],
        faces: &[ObjFace],
        material_id: Option<usize>,
    ) -> Mesh {
        let mut vertex_map: HashMap<ObjCorner, usize> = HashMap::new();
        let mut indices = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        for face in faces {
            if face.len() < 3 {
                continue;
            }

            for i in 1..face.len() - 1 {
                for corner in [face[0], face[i], face[i + 1]] {
                    let (v, vt, vn) = corner;

                    let index = *vertex_map.entry(corner).or_insert_with(|| {
                        let index = vertices.len();

                        let new_vertex = Vertex::new(
                            positions[v],
                            tex_coords.get(vt).copied().unwrap_or([0.0, 0.0]),
                            normals.get(vn).copied().unwrap_or([0.0, 0.0, 0.0]),
                        );

                        vertices.push(new_vertex);
                        index
                    });

                    indices.push(index);
                }
            }
        }

        Mesh::new(vertices, indices, material_id)
    }

    pub fn convert_to_mesh(&self, obj: ObjMesh) -> Model {
        let mut model = Model::new(obj.materials);

        for object in obj.objects {
            for group in object.groups {
                let mesh = self.build_mesh_from_obj(
                    &obj.position,
                    &obj.tex_coords,
                    &obj.normal,
                    &group.faces,
                    group.material_id,
                );

                model.meshes.push(mesh);
            }
        }

        model
    }
}