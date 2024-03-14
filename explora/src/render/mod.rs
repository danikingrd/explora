pub mod atlas;
pub mod buffer;
pub mod mesh;
pub mod png_utils;
pub mod texture;
pub mod voxels;

use std::sync::Arc;

use common::math::{Mat4f, Vec3};
use pollster::FutureExt;
use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
use winit::window::Window;

use crate::{
    render::{atlas::Atlas, buffer::Buffer, texture::Texture, voxels::Voxels},
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

/// Manages the rendering of the application.
pub struct Renderer {
    /// Surface on which the renderer will draw.
    surface: wgpu::Surface<'static>,
    /// The Logical Device, used for interacting with the GPU.
    device: wgpu::Device,
    /// A Queue handle. Used for command submission.
    queue: wgpu::Queue,
    /// The surface configuration details.
    config: wgpu::SurfaceConfiguration,
    /// Uniforms available on the GPU.
    uniforms_buffer: Buffer<Uniforms>,
    /// Common Bind Groups
    common_bg: wgpu::BindGroup,
    /// Block texture atlas.
    atlas: Atlas,
    /// Terrain Depth Texture
    depth_texture: Texture,
    /// Voxel Renderer
    voxels: Voxels,
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

        let voxels = Voxels::new(&device, &common_bind_group_layout, &config);
        tracing::info!("Renderer initialized.");

        Self {
            surface,
            device,
            queue,
            config,
            uniforms_buffer,
            common_bg: common_bind_group,
            atlas,
            depth_texture,
            voxels,
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

            self.voxels.draw(&mut render_pass, &self.common_bg);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
