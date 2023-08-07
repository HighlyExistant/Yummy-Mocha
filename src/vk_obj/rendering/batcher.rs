use ash::vk;

use std::sync::Arc;
use crate::vk_obj::{buffer, device::ReplacingDevice};

use super::mesh::{VulkanIndexable, Mesh, Vertex};

pub struct RenderBatch<V: Vertex, I: VulkanIndexable> {
    pub vertices: buffer::raw::Buffer<V>,
    pub indices: buffer::raw::Buffer<I>,
    pub index_count: u32,
}

impl<V: Vertex, I: VulkanIndexable> RenderBatch<V, I> {
    fn new(device: Arc<ReplacingDevice>, vertices: Vec<V>, indices: Vec<I>) -> Self {
        let index_count = indices.len() as u32;
        Self { 
            vertices: buffer::raw::Buffer::from_vec(device.clone(), vk::BufferUsageFlags::VERTEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, &vertices), 
            indices: buffer::raw::Buffer::from_vec(device.clone(), vk::BufferUsageFlags::INDEX_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, &indices),
            index_count,
        }
    }
}
pub struct RenderBatchBuilder<V: Vertex, I: VulkanIndexable> {
    vertices: Vec<V>, 
    indices: Vec<I>,
    index_offset: I,
}
impl<V: Vertex, I: VulkanIndexable> RenderBatchBuilder<V, I> {
    pub fn new() -> Self {
        Self { vertices: vec![], indices: vec![], index_offset: I::zero() }
    }
    pub fn push(mut self, mesh: &dyn Mesh<V, I>) -> Self {
        let mut vertices = mesh.vertices();

        self.vertices.append(&mut vertices);

        let mesh_indices = mesh.indices();
        let mut indices = mesh_indices.iter().map(|index| {
            *index + self.index_offset
        }).collect();
        self.indices.append(&mut indices);

        let mut casted: *mut usize = &self.index_offset as *const _ as *mut _;
        unsafe { *casted +=  indices.len() };
        self
    }
    pub fn build(mut self, device: Arc<ReplacingDevice>) -> RenderBatch<V, I> {
        RenderBatch::<V, I>::new(device, self.vertices, self.indices)
    }
}