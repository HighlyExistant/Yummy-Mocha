#![allow(unused)]
use ash::vk;
use ash::vk::StencilOpState;
use ash::vk::VertexInputAttributeDescription;
use ash::vk::VertexInputBindingDescription;
use crate::vk_obj::device::ReplacingDevice;
use crate::vk_obj::pipelines;
use crate::vk_obj::device;
use crate::vk_obj::rendering::mesh::Vertex;
use std::marker::PhantomData;
use std::ops::Index;
use std::sync::Arc;

use super::create_shader_module;

pub struct GraphicsPipelines {
    pub pipelines: Vec<vk::Pipeline>,
    mods: Vec<vk::ShaderModule>,
    device: std::sync::Arc<ReplacingDevice>,
}
#[derive(Default)]
pub struct GraphicsPipelineInfo
{
    pub vertex_filepath: String,
    pub vertex_entry: String,
    pub fragment_filepath: String,
    pub fragment_entry: String,
    pub culling: vk::CullModeFlags,
    pub layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
    pub subpass: u32,
}
#[derive(Default, Clone)]
struct GraphicsPipelineData {
    flags: Vec<vk::PipelineCreateFlags>,
    stage_count: Vec<u32>,
    stages: Vec<vk::PipelineShaderStageCreateInfo>,
    vertex_input_state: Vec<vk::PipelineVertexInputStateCreateInfo>,
    viewport_state: Vec<vk::PipelineViewportStateCreateInfo>,
    rasterization_state: Vec<vk::PipelineRasterizationStateCreateInfo>,
    dynamic_state: Vec<vk::PipelineDynamicStateCreateInfo>,
    layout: Vec<vk::PipelineLayout>,
    render_pass: Vec<vk::RenderPass>,
    subpass: Vec<u32>,
    input_assembly_state: Vec<vk::PipelineInputAssemblyStateCreateInfo>,
    multisample_state: Vec<vk::PipelineMultisampleStateCreateInfo>,
    depth_stencil: Vec<vk::PipelineDepthStencilStateCreateInfo>,
    color_blend: Vec<vk::PipelineColorBlendAttachmentState>,
    color_blend_state: Vec<vk::PipelineColorBlendStateCreateInfo>,
    shader_stages: Vec<Vec<vk::PipelineShaderStageCreateInfo>>
}
#[derive(Default)]
pub struct GraphicsPipelineBuilder {
    // data: Box<GraphicsPipelineData>,
    data: GraphicsPipelineData,
    unique_modules: Vec<vk::ShaderModule>,
    create_infos: Vec<vk::GraphicsPipelineCreateInfo>
}
impl GraphicsPipelineBuilder {
    pub fn new() -> Self {
        GraphicsPipelineBuilder::default()
    }
    pub fn subpass(mut self, subpass: u32) -> Self {self.data.subpass.push(subpass); self }
    pub fn rasterization(mut self, polygon_mode: vk::PolygonMode, culling: vk::CullModeFlags) -> Self {
        self.data.rasterization_state.push(vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: 0,  // VK_FALSE
            rasterizer_discard_enable: 0,  // VK_FALSE
            polygon_mode: polygon_mode,
            line_width: 1.0,
            cull_mode: culling,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: 0, // VK_FALSE
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,
            ..Default::default()
        });
        self
    }
    pub fn dynamic_states(mut self, dynamic: &[vk::DynamicState]) -> Self {
        self.data.dynamic_state.push(vk::PipelineDynamicStateCreateInfo {
            p_dynamic_states: dynamic.as_ptr(),
            dynamic_state_count: dynamic.len() as u32,
            ..Default::default()
        });
        self
    }
    pub fn add_unique_shader_module(mut self, modules: vk::ShaderModule) -> Self {
        self.unique_modules.push(modules);
        self
    }
    pub fn add_shader_stage(mut self, shader_stages:  Vec<vk::PipelineShaderStageCreateInfo>) -> Self {
        self.data.shader_stages.push(shader_stages);
        self
    }
    pub fn vertex_input_state<V: Vertex>(mut self, binding: &VertexInputBindingDescription, attribute: &Vec<VertexInputAttributeDescription>) -> Self {
        self.data.vertex_input_state.push(vk::PipelineVertexInputStateCreateInfo {
            vertex_attribute_description_count: attribute.len() as u32,
            p_vertex_attribute_descriptions: attribute.as_ptr(),
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: binding,
            ..Default::default()
        });
        self
    }
    pub fn pipeline_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.data.layout.push(layout);
        self
    }
    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> Self {
        self.data.render_pass.push(render_pass);
        self
    }
    pub fn clear_shader_stages(mut self) -> Self {
        self.data.shader_stages.clear();
        self
    }
    pub fn push_info(mut self) -> Self {
        self.data.input_assembly_state.push(vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: 0, // VK_FALSE
            ..Default::default()
        });

        self.data.viewport_state.push(vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            scissor_count: 1,
            ..Default::default()
        });
        self.data.multisample_state.push(vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: 0, // VK_FALSE
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            alpha_to_coverage_enable: 0, // VK_FALSE
            alpha_to_one_enable: 0, // VK_FALSE
            ..Default::default()
        });
        self.data.depth_stencil.push(vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1, // VK_TRUE
            depth_write_enable: 1, // VK_TRUE
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: 0, // VK_FALSE
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            stencil_test_enable: 0, // VK_FALSE
            front: StencilOpState::default(),
            back: StencilOpState::default(),
            ..Default::default()
        });

        self.data.color_blend.push(vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            blend_enable: 1, // VK_TRUE
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            ..Default::default()
        });

        self.data.color_blend_state.push(vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: 0, // VK_FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: self.data.color_blend.last().unwrap(),

            ..Default::default()
        });
        // self.pushed_data.push(self.data.clone());
        let create_info = vk::GraphicsPipelineCreateInfo {
            stage_count: self.data.shader_stages.last().unwrap().len() as u32,
            p_stages: self.data.shader_stages.last().unwrap().as_ptr(),
            p_vertex_input_state: self.data.vertex_input_state.last().unwrap(),
            p_input_assembly_state: self.data.input_assembly_state.last().unwrap(),
            p_viewport_state: self.data.viewport_state.last().unwrap(),
            p_rasterization_state: self.data.rasterization_state.last().unwrap(),
            p_multisample_state: self.data.multisample_state.last().unwrap(),
            p_depth_stencil_state: self.data.depth_stencil.last().unwrap(),
            p_color_blend_state: self.data.color_blend_state.last().unwrap(),
            p_dynamic_state: self.data.dynamic_state.last().unwrap(),
            layout: *self.data.layout.last().unwrap(),
            render_pass: *self.data.render_pass.last().unwrap(),
            subpass: *self.data.subpass.last().unwrap(),
            base_pipeline_index: -1,
    
            ..Default::default()
        };
        self.create_infos.push(create_info);
        self
    }
    pub fn build(self, device: std::sync::Arc<ReplacingDevice>, cache: vk::PipelineCache) -> GraphicsPipelines{
        let pipelines = unsafe { device.device.create_graphics_pipelines(cache, &self.create_infos, None).unwrap() };
        GraphicsPipelines { pipelines, device, mods: self.unique_modules }
    }
}

impl GraphicsPipelines {
    pub fn new<T>(device: Arc<ReplacingDevice>, info: &GraphicsPipelineInfo) -> Self
    where T: Vertex {
        let vertex = create_shader_module(&device.device, &info.vertex_filepath);
        let fragment = create_shader_module(&device.device, &info.fragment_filepath);
        let stages = vec![
            vk::PipelineShaderStageCreateInfo {
            module: vertex,
            stage: vk::ShaderStageFlags::VERTEX,
            p_name: info.vertex_entry.as_ptr() as *const i8,
            ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
            module: fragment,
            stage: vk::ShaderStageFlags::FRAGMENT,
            p_name: info.fragment_entry.as_ptr() as *const i8,
            ..Default::default()
            }
        ];
        {
            let binding = T::binding_description();
            let attribute = T::attribute_description();
            GraphicsPipelineBuilder::new()
            .add_unique_shader_module(vertex)
            .add_unique_shader_module(fragment)
            .add_shader_stage(stages.clone())
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .pipeline_layout(info.layout)
            .render_pass(info.renderpass)
            .subpass(info.subpass)
            .rasterization(vk::PolygonMode::FILL, vk::CullModeFlags::NONE)
            .vertex_input_state::<T>(&binding, &attribute)
            .push_info()
            // .rasterization(vk::PolygonMode::LINE, vk::CullModeFlags::NONE)
            // .push_info()
            .build(device.clone(), vk::PipelineCache::null())
        }
    }

    pub fn builder() -> GraphicsPipelineBuilder {
        GraphicsPipelineBuilder::default()
    }
}
impl Index<usize> for GraphicsPipelines {
    fn index(&self, index: usize) -> &Self::Output {
        &self.pipelines[index]
    }
    type Output = vk::Pipeline;

}

impl Drop for GraphicsPipelines {
    fn drop(&mut self) {
        unsafe {
            for shader in &self.mods {
                self.device.device.destroy_shader_module(*shader, None);
            }
            for pipeline in &self.pipelines {
                self.device.device.destroy_pipeline(*pipeline, None);
            }
        }
    }
}