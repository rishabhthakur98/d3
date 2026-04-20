/// #[repr(C)] forces Rust to lay out this struct in memory exactly as declared.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3], 
    pub color: [f32; 4],    
    pub normal: [f32; 3],   
    pub uv: [f32; 2],       
}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
            normal: [0.0, 1.0, 0.0], 
            uv: [0.0, 0.0],          
        }
    }

    pub fn with_uv(mut self, uv: [f32; 2]) -> Self {
        self.uv = uv;
        self
    }
}