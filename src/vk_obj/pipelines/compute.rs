use ash::vk;

use crate::vk_obj::device::ReplacingDevice;
use std::sync::Arc;

use super::create_shader_module;
pub struct ComputePipelines {
    device: Arc<ReplacingDevice>,
    mods: Vec<vk::ShaderModule>,
    pub pipelines: Vec<vk::Pipeline>,
}
pub struct ComputePipelinesBuilder {
    stage:  Vec<vk::PipelineShaderStageCreateInfo>,
    unique_modules: Vec<vk::ShaderModule>,
    info: Vec<vk::ComputePipelineCreateInfo>,
    layout: Vec<vk::PipelineLayout>
}

impl ComputePipelinesBuilder {
    pub fn new() -> Self {
        Self { stage: vec![], layout: vec![], info: vec![], unique_modules: vec![] }
    }
    pub fn add_shader_stage(mut self, stage:  vk::PipelineShaderStageCreateInfo) -> Self {
        self.stage.push(stage);
        self
    }
    pub fn add_unique_shader_module(mut self, modules: vk::ShaderModule) -> Self {
        self.unique_modules.push(modules);
        self
    }
    pub fn pipeline_layout(mut self, layout: vk::PipelineLayout) -> Self {
        self.layout.push(layout);
        self
    }
    pub fn push_info(mut self) -> Self {
        let create_info = vk::ComputePipelineCreateInfo {
            layout: *self.layout.last().unwrap(),
            stage: *self.stage.last().unwrap(),
            ..Default::default()
        };
        self.info.push(create_info);
        self
    }
    pub fn build(self, device: Arc<ReplacingDevice>, cache: vk::PipelineCache) -> ComputePipelines {
        let pipelines = unsafe { device.device.create_compute_pipelines(cache, &self.info, None).unwrap() };
        ComputePipelines { pipelines, device: device.clone(), mods: self.unique_modules }
    }
}

impl ComputePipelines {
    pub fn new(device: Arc<ReplacingDevice>, layout: vk::PipelineLayout, cache: vk::PipelineCache, shader: &str, entry: &str) -> Self {
        let shader_module = create_shader_module(&device.device, shader);
        let stage = vk::PipelineShaderStageCreateInfo {
            module: shader_module,
            p_name: entry.as_ptr() as *const i8,
            stage: vk::ShaderStageFlags::COMPUTE,
            ..Default::default()
        };
        ComputePipelinesBuilder::new().pipeline_layout(layout).add_unique_shader_module(shader_module).add_shader_stage(stage).push_info().build(device.clone(), cache)
    }
}

impl Drop for ComputePipelines {
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