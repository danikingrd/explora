use common::{chunk::Chunk, math::Vec2};

use super::{atlas::Atlas, buffer::Buffer, mesh, texture::Texture, Vertex};

pub struct Voxels {
    /// Terrain render pipeline
    render_pipeline: wgpu::RenderPipeline,
    // /// Terrain geometry
    chunk_meshes: Vec<Buffer<Vertex>>,
    /// Terrain indices
    index_buffer: Buffer<u32>,
}

impl Voxels {
    pub fn new(
        device: &wgpu::Device,
        common_bg_layout: &wgpu::BindGroupLayout,
        config: &wgpu::SurfaceConfiguration,
        atlas: &Atlas,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/voxels.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&common_bg_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Test geometry
        let mut chunk_generation = vec![];
        for x in 0..3 {
            for z in 0..3 {
                chunk_generation.push((Vec2::new(x, z), Chunk::flat()));
            }
        }

        let mut chunk_meshes = vec![];
        let mut vertex_count = 0;

        for (pos, chunk) in chunk_generation {
            let mut chunk_mesh = vec![];
            mesh::create_chunk_mesh(&chunk, &mut chunk_mesh, pos, atlas);
            chunk_meshes.push(Buffer::new(device, wgpu::BufferUsages::VERTEX, &chunk_mesh));
            vertex_count += chunk_mesh.len() as u32;
        }

        let index_buffer = Buffer::new(
            device,
            wgpu::BufferUsages::INDEX,
            &compute_voxel_indices(vertex_count as usize),
        );

        Self {
            render_pipeline,
            chunk_meshes,
            index_buffer,
        }
    }

    pub fn draw<'a>(
        &'a mut self,
        frame: &mut wgpu::RenderPass<'a>,
        common_bg: &'a wgpu::BindGroup,
    ) {
        frame.set_pipeline(&self.render_pipeline);
        frame.set_bind_group(0, common_bg, &[]);
        frame.set_index_buffer(self.index_buffer.slice(), wgpu::IndexFormat::Uint32);
        for chunk_mesh in &self.chunk_meshes {
            frame.set_vertex_buffer(0, chunk_mesh.slice());
            frame.draw_indexed(0..chunk_mesh.len() / 4 * 6, 0, 0..1);
        }
    }
}

fn compute_voxel_indices(number_of_vertices: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity(number_of_vertices * 6 / 4);
    for i in 0..number_of_vertices / 4 {
        let offset = i as u32 * 4;
        indices.extend_from_slice(&[
            offset,
            offset + 1,
            offset + 2,
            offset + 2,
            offset + 3,
            offset,
        ]);
    }
    indices
}
