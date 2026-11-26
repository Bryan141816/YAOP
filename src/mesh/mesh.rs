use crate::mesh::Vertex;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub material_id: Option<usize>,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<usize>,
        material_id: Option<usize>,
    ) -> Self {
        Self {
            vertices,
            indices,
            material_id,
        }
    }
}