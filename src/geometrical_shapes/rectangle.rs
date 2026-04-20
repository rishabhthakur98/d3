// src/geometrical_shapes/rectangle.rs
use crate::geometrical_shapes::mesh::Mesh; // FIXED IMPORT
use crate::geometrical_shapes::triangle::Vertex;

pub struct Rectangle;

impl Rectangle {
    pub fn generate(width: f32, depth: f32, color: [f32; 4]) -> Mesh {
        let half_w = width / 2.0;
        let half_d = depth / 2.0;

        let vertices = vec![
            Vertex::new([-half_w, 0.0, -half_d], color).with_uv([0.0, 0.0]),
            Vertex::new([half_w, 0.0, -half_d], color).with_uv([1.0, 0.0]),
            Vertex::new([half_w, 0.0, half_d], color).with_uv([1.0, 1.0]),
            Vertex::new([-half_w, 0.0, half_d], color).with_uv([0.0, 1.0]),
        ];

        let indices = vec![0, 3, 2, 2, 1, 0];
        Mesh { vertices, indices }
    }
}