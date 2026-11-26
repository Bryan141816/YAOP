#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}
impl Vertex {
    pub fn new(    
        position: [f32; 3],
        tex_coord: [f32; 2],
        normal: [f32; 3]
    ) -> Self {
        Self {
            position,
            tex_coord,
            normal
        }
    }
}