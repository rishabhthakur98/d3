use crate::geometrical_shapes::mesh::Mesh;
use crate::geometrical_shapes::triangle::Vertex;

pub struct RiverMesh;

impl RiverMesh {
    /// Generates a long strip representing a river.
    /// UV.x = 0.0 to 1.0 (Left bank to Right bank) -> Used for shoreline fading.
    /// UV.y = 0.0 to Length (Along the river) -> Used for flow scrolling.
    pub fn generate(length: f32, width: f32, res_l: u32, res_w: u32, color: [f32; 4]) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_w = width / 2.0;
        let step_l = length / (res_l as f32);
        let step_w = width / (res_w as f32);

        for z in 0..=res_l {
            for x in 0..=res_w {
                let px = -half_w + (x as f32) * step_w;
                let pz = (z as f32) * step_l; // River flows down +Z
                
                // U is 0.0 to 1.0 across the width. V stretches along the length.
                let uv = [
                    (x as f32) / (res_w as f32),
                    (z as f32) / 5.0 // Tile the texture/waves every 5 units
                ];

                vertices.push(Vertex::new([px, 0.0, pz], color).with_uv(uv));
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

        Mesh { vertices, indices }
    }
}