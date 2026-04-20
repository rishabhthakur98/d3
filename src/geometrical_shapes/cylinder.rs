use crate::geometrical_shapes::mesh::Mesh;
use crate::geometrical_shapes::triangle::Vertex;
use std::f32::consts::PI;

pub struct Cylinder;

impl Cylinder {
    pub fn generate(radius: f32, height: f32, segments: u32, color: [f32; 4]) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let half_h = height / 2.0;

        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, half_h, 0.0], color));
        
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, -half_h, 0.0], color));

        let base_idx = vertices.len() as u32;
        for i in 0..segments {
            let angle = (i as f32) * 2.0 * PI / (segments as f32);
            let x = radius * angle.cos();
            let z = radius * angle.sin();

            vertices.push(Vertex::new([x, half_h, z], color).with_uv([x, z]));
            vertices.push(Vertex::new([x, -half_h, z], color).with_uv([x, z]));
        }

        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let top0 = base_idx + i * 2;
            let bot0 = base_idx + i * 2 + 1;
            let top1 = base_idx + next_i * 2;
            let bot1 = base_idx + next_i * 2 + 1;

            // Side quad
            indices.extend_from_slice(&[top0, bot0, bot1, top0, bot1, top1]);
            // Top cap
            indices.extend_from_slice(&[top_center_idx, top1, top0]);
            // Bottom cap
            indices.extend_from_slice(&[bottom_center_idx, bot0, bot1]);
        }

        Mesh { vertices, indices }
    }
}