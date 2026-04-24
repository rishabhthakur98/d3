// src/water/river_mesh.rs

use crate::assets::model::{Mesh, Vertex, MaterialConfig};

pub struct RiverMesh;

impl RiverMesh {
    pub fn generate(length: f32, width: f32, res_l: u32, res_w: u32, color: [f32; 4]) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_w = width / 2.0;
        let step_l = length / (res_l as f32);
        let step_w = width / (res_w as f32);

        for z in 0..=res_l {
            for x in 0..=res_w {
                let px = -half_w + (x as f32) * step_w;
                let pz = (z as f32) * step_l; 
                
                let uv = [
                    (x as f32) / (res_w as f32),
                    (z as f32) / 5.0 
                ];

                vertices.push(Vertex {
                    position: [px, 0.0, pz], 
                    color, 
                    normal: [0.0, 1.0, 0.0],
                    uv
                });
            }
        }

        for z in 0..res_l {
            for x in 0..res_w {
                let tl = z * (res_w + 1) + x;
                let tr = tl + 1;
                let bl = (z + 1) * (res_w + 1) + x;
                let br = bl + 1;

                indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
            }
        }

        Mesh { 
            vertices, 
            indices,
            // Supply the new required material field with a safe default
            material: MaterialConfig::default() 
        }
    }
}