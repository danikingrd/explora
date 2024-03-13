pub mod atlas;
pub mod buffer;
pub mod png_utils;
pub mod texture;

use std::sync::Arc;

use common::math::{Mat4f, Vec3};
use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
use winit::window::Window;

use crate::{
    render::{atlas::Atlas, buffer::Buffer, texture::Texture},
    scene::Scene,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    atlas_size: u32,
    atlas_tile_count: u32,
    _padding: [f32; 2],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            proj: Mat4f::identity().into_col_arrays(),
            view: Mat4f::identity().into_col_arrays(),
            atlas_size: 0,
            atlas_tile_count: 0,
            _padding: [0.0; 2],
        }
    }
}

impl Uniforms {
    pub fn new(proj: Mat4f, view: Mat4f, atlas_size: u32, atlas_tile_count: u32) -> Self {
        Self {
            proj: proj.into_col_arrays(),
            view: view.into_col_arrays(),
            atlas_size,
            atlas_tile_count,
            _padding: [0.0; 2],
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct Vertex {
    pos: [f32; 3],
    texture_id: u32,
}

impl Vertex {
    pub fn new(v: Vec3<f32>, texture_id: u32) -> Self {
        Self {
            pos: v.into_array(),
            texture_id,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32];
        wgpu::VertexBufferLayout {
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u32>,
    uniforms_buffer: Buffer<Uniforms>,
    common_bg: wgpu::BindGroup,
    atlas: Atlas,
    depth_texture: Texture,
}

impl Renderer {
    #[allow(clippy::vec_init_then_push)]
    pub fn new(platform: &Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(platform.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        let (width, height) = platform.inner_size().into();
        let config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/terrain.wgsl").into(),
            ),
        });

        let uniforms_buffer = Buffer::new(
            &device,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[Uniforms::default()],
        );
        let atlas = Atlas::pack_textures("assets/textures/block/").unwrap();
        let atlas_texture = Texture::new(&device, &queue, &atlas.image);
        let depth_texture = Texture::depth(&device, config.width, config.height);
        let common_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Common Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let common_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Common Bind Group"),
            layout: &common_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas_texture.sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&common_bind_group_layout],
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
                format: texture::Texture::DEPTH_FORMAT,
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

        let mut cube_mesh = vec![];
        let mut indices = vec![];

        for z in 0..3 {
            let offset = Vec3::new(0.0, z as f32, z as f32);
            // North
            let north = 1;
            cube_mesh.push(Vertex::new(Vec3::unit_y() + offset, north));
            cube_mesh.push(Vertex::new(Vec3::zero() + offset, north));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + offset, north));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_y() + offset, north));
            // South
            let south = 1;
            cube_mesh.push(Vertex::new(
                Vec3::unit_x() + Vec3::unit_y() + Vec3::unit_z() + offset,
                south,
            ));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_z() + offset, south));
            cube_mesh.push(Vertex::new(Vec3::zero() + Vec3::unit_z() + offset, south));
            cube_mesh.push(Vertex::new(Vec3::unit_y() + Vec3::unit_z() + offset, south));

            // East
            let east = 1;
            cube_mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_y() + offset, east));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + offset, east));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_z() + offset, east));
            cube_mesh.push(Vertex::new(
                Vec3::unit_x() + Vec3::unit_z() + Vec3::unit_y() + offset,
                east,
            ));

            // West
            let west = 1;
            cube_mesh.push(Vertex::new(Vec3::unit_z() + Vec3::unit_y() + offset, west));
            cube_mesh.push(Vertex::new(Vec3::unit_z() + offset, west));
            cube_mesh.push(Vertex::new(Vec3::zero() + offset, west));
            cube_mesh.push(Vertex::new(Vec3::unit_y() + offset, west));

            // Top
            let top = 2;
            cube_mesh.push(Vertex::new(Vec3::unit_z() + Vec3::unit_y() + offset, top));
            cube_mesh.push(Vertex::new(Vec3::unit_y() + offset, top));
            cube_mesh.push(Vertex::new(Vec3::unit_y() + Vec3::unit_x() + offset, top));
            cube_mesh.push(Vertex::new(
                Vec3::unit_y() + Vec3::unit_x() + Vec3::unit_z() + offset,
                top,
            ));

            // Bottom
            let bottom = 0;
            cube_mesh.push(Vertex::new(Vec3::zero() + offset, bottom));
            cube_mesh.push(Vertex::new(Vec3::unit_z() + offset, bottom));
            cube_mesh.push(Vertex::new(
                Vec3::unit_z() + Vec3::unit_x() + offset,
                bottom,
            ));
            cube_mesh.push(Vertex::new(Vec3::unit_x() + offset, bottom));

            let mut indices_temp = vec![];
            let mut quad = z * 6 * 4;

            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);
            quad += 4;
            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);
            quad += 4;
            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);
            quad += 4;
            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);
            quad += 4;
            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);
            quad += 4;
            indices_temp.extend_from_slice(&[quad, quad + 1, quad + 2, quad + 2, quad + 3, quad]);

            indices.extend_from_slice(&indices_temp);
        }

        let vertex_buffer = Buffer::new(&device, wgpu::BufferUsages::VERTEX, &cube_mesh);
        let index_buffer = Buffer::new(&device, wgpu::BufferUsages::INDEX, &indices);

        tracing::info!("Renderer initialized.");

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniforms_buffer,
            common_bg: common_bind_group,
            atlas,
            depth_texture,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::depth(&self.device, width, height);
    }

    pub fn render(&mut self, scene: &mut Scene) {
        let matrices = scene.camera_matrices();
        self.uniforms_buffer.write(
            &self.queue,
            &[Uniforms::new(
                matrices.proj,
                matrices.view,
                self.atlas.image.width,
                self.atlas.tile_size as u32,
            )],
        );

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.common_bg, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice());
            render_pass.set_index_buffer(self.index_buffer.slice(), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_buffer.len(), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
