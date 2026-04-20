use crate::geometrical_shapes::triangle::Vertex;

/// A universal mesh format separating vertices and indices to save GPU memory.
#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}