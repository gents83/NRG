use nrg_math::{get_translation_rotation_scale, Matrix4};
use nrg_math::{Vector4, Zero};

use crate::{Mesh, MeshInstance, PipelineId};

use super::data_formats::*;
use super::device::*;
use super::render_pass::*;
use super::shader::*;
use super::texture::*;

#[derive(Clone)]
pub struct Pipeline {
    pub inner: crate::api::backend::pipeline::Pipeline,
    id: PipelineId,
    mesh: Mesh,
    vertex_count: u32,
    index_count: u32,
    instance_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
}
unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

impl Pipeline {
    pub fn id(&self) -> PipelineId {
        self.id
    }

    pub fn create(
        device: &Device,
        id: PipelineId,
        data: &PipelineData,
        render_pass: &RenderPass,
    ) -> Pipeline {
        //TODO pipeline could be reused - while instance should be unique
        let mut pipeline = crate::api::backend::pipeline::Pipeline::create(&device.inner);

        pipeline
            .set_shader(ShaderType::Vertex, data.vertex_shader.as_path())
            .set_shader(ShaderType::Fragment, data.fragment_shader.as_path());
        if !data.tcs_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(ShaderType::TessellationControl, data.tcs_shader.as_path());
        }
        if !data.tes_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(
                ShaderType::TessellationEvaluation,
                data.tes_shader.as_path(),
            );
        }
        if !data.geometry_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(ShaderType::Geometry, data.geometry_shader.as_path());
        }
        pipeline.build(&device.inner, &render_pass.get_pass());

        Pipeline {
            inner: pipeline,
            id,
            mesh: Mesh::create(device),
            vertex_count: 0,
            index_count: 0,
            instance_count: 0,
            instance_data: Vec::new(),
            instance_commands: Vec::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.instance_commands.is_empty() || self.vertex_count.is_zero()
    }
    pub fn get_instance_data(&self) -> &Vec<InstanceData> {
        &self.instance_data
    }
    pub fn get_instance_commands(&self) -> &Vec<InstanceCommand> {
        &self.instance_commands
    }
    pub fn get_instance_count(&self) -> usize {
        self.instance_count
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn prepare(&mut self) -> &mut Self {
        self.vertex_count = 0;
        self.index_count = 0;
        self.instance_count = 0;
        self
    }

    pub fn begin(&mut self) -> &mut Self {
        self.inner
            .bind(&self.instance_commands, &self.instance_data)
            .bind_descriptors();
        self
    }

    pub fn update_runtime_data(&self, view: &Matrix4, proj: &Matrix4) -> &Self {
        self.inner.update_constant_data(view, proj);
        self.inner.update_uniform_buffer(view, proj);
        self
    }
    pub fn update_descriptor_sets(&self, textures: &[TextureAtlas]) -> &Self {
        self.inner.update_descriptor_sets(textures);
        self
    }

    pub fn bind_indirect(&mut self) -> &mut Self {
        self.inner.bind_indirect();
        self
    }
    pub fn bind_vertices(&mut self) -> &mut Self {
        self.mesh.bind_vertices();
        self
    }
    pub fn bind_indices(&mut self) -> &mut Self {
        self.mesh.bind_indices();
        self
    }

    pub fn draw_indirect(&mut self) -> &mut Self {
        self.inner.draw_indirect(self.instance_count);
        self
    }

    pub fn end(&mut self) -> &mut Self {
        self
    }

    pub fn add_mesh_instance(
        &mut self,
        mesh_instance: &MeshInstance,
        diffuse_color: Vector4,
        diffuse_texture_index: i32,
        diffuse_layer_index: i32,
        outline_color: Vector4,
    ) -> &mut Self {
        let mesh_data_ref = self.mesh.bind_at_index(
            &mesh_instance.get_data().vertices,
            self.vertex_count,
            &mesh_instance.get_data().indices,
            self.index_count,
        );

        self.vertex_count += mesh_instance.get_data().vertices.len() as u32;
        self.index_count += mesh_instance.get_data().indices.len() as u32;

        let command = InstanceCommand {
            mesh_index: self.instance_count,
            mesh_data_ref,
        };
        let (position, rotation, scale) = get_translation_rotation_scale(mesh_instance.transform());

        let data = InstanceData {
            position,
            rotation,
            scale,
            draw_area: mesh_instance.draw_area(),
            diffuse_color,
            diffuse_texture_index,
            diffuse_layer_index,
            outline_color,
        };
        if self.instance_count >= self.instance_commands.len() {
            self.instance_commands.push(command);
            self.instance_data.push(data);
        } else {
            self.instance_commands[self.instance_count] = command;
            self.instance_data[self.instance_count] = data;
        }
        self.instance_count += 1;
        self
    }
}
