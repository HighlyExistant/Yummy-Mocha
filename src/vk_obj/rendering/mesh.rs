use ash::vk;
use num_traits;
use std::sync::Arc;
use crate::{vk_obj::{buffer::{raw, self}, device::{self}}, camera::Camera};
pub trait VulkanIndexable: num_traits::Num + core::clone::Clone + num_traits::AsPrimitive<u8> + num_traits::AsPrimitive<u16> + num_traits::AsPrimitive<u32> + num_traits::AsPrimitive<usize> + core::clone::Clone {}
pub trait Vertex: Sized + Copy + Clone {
    fn binding_description() -> vk::VertexInputBindingDescription;
    fn attribute_description() -> Vec<vk::VertexInputAttributeDescription>;
}

impl VulkanIndexable for u8 {}
impl VulkanIndexable for u16 {}
impl VulkanIndexable for u32 {}
pub trait Mesh<V: Vertex, I: VulkanIndexable> {
    fn vertices(&self) -> Vec<V>;
    fn indices(&self) -> Vec<I>;
}