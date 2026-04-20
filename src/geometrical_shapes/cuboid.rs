// src/geometrical_shapes/cuboid.rs
use crate::geometrical_shapes::mesh::Mesh; // FIXED IMPORT
use crate::geometrical_shapes::triangle::Vertex;

pub struct Cuboid;

impl Cuboid {
    pub fn generate(width: f32, height: f32, depth: f32, color: [f32; 4]) -> Mesh {
        let w = width / 2.0;
        let h = height / 2.0;
        let d = depth / 2.0;

        let vertices = vec![
            Vertex::new([-w, -h, -d], color), 
            Vertex::new([ w, -h, -d], color), 
            Vertex::new([ w,  h, -d], color), 
            Vertex::new([-w,  h, -d], color), 
            Vertex::new([-w, -h,  d], color), 
            Vertex::new([ w, -h,  d], color), 
            Vertex::new([ w,  h,  d], color), 
            Vertex::new([-w,  h,  d], color), 
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,
            5, 4, 7, 7, 6, 5,
            4, 0, 3, 3, 7, 4,
            1, 5, 6, 6, 2, 1,
            3, 2, 6, 6, 7, 3,
            4, 5, 1, 1, 0, 4,
        ];

        Mesh { vertices, indices }
    }
}