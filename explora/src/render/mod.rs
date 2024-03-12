pub mod buffer;

use std::sync::Arc;

use common::math::{Mat4f, Vec3};
use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
use winit::window::Window;

use crate::{render::buffer::Buffer, scene::Scene};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            proj: Mat4f::identity().into_col_arrays(),
            view: Mat4f::identity().into_col_arrays(),
        }
    }
}

impl Uniforms {
    pub fn new(proj: Mat4f, view: Mat4f) -> Self {
        Self {
            proj: proj.into_col_arrays(),
            view: view.into_col_arrays(),
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct Vertex {
    pos: [f32; 3],
}

impl Vertex {
    pub fn new(v: Vec3<f32>) -> Self {
        Self {
            pos: v.into_array(),
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];
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
}

impl Renderer {
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

        let common_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Common Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let common_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Common Bind Group"),
            layout: &common_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let quad = [
            Vertex::new(Vec3::new(-0.5, 0.5, 0.0)),
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0)),
            Vertex::new(Vec3::new(0.5, -0.5, 0.0)),
            Vertex::new(Vec3::new(0.5, 0.5, 0.0)),
        ];

        let indices = [0u32, 1, 2, 2, 3, 0];

        let vertex_buffer = Buffer::new(&device, wgpu::BufferUsages::VERTEX, &quad);
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
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, scene: &mut Scene) {
        let matrices = scene.camera_matrices();
        self.uniforms_buffer
            .write(&self.queue, &[Uniforms::new(matrices.proj, matrices.view)]);
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.common_bg, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice());
            render_pass.set_index_buffer(self.index_buffer.slice(), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
