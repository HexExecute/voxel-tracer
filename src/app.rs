use std::{time::Instant, num::NonZeroU64};

use bytemuck::Contiguous;
use shared::ShaderConstants;
use wgpu::util::DeviceExt;
use winit::{window::Window, event::WindowEvent};

use crate::svo::SparseVoxelOctree;

pub struct State {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    start_time: Instant,

    render_pipeline: wgpu::RenderPipeline,
    shader_constants: ShaderConstants,

    // svo: SparseVoxelOctree,
    // node_buffer: wgpu::Buffer,
    // voxel_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let svo = SparseVoxelOctree::new(3);
        let packed_svo = svo.pack();

        // dbg!(&packed_svo.1odes);

        let node_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("node_buffer"),
            contents: bytemuck::cast_slice(&packed_svo.nodes),
            usage: wgpu::BufferUsages::STORAGE
        });

        let voxel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("voxel_buffer"),
            contents: bytemuck::cast_slice(&packed_svo.voxels),
            usage: wgpu::BufferUsages::STORAGE
        });

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!(env!("shader.spv"))) };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: NonZeroU64::from_integer(32) },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: NonZeroU64::from_integer(16) },
                    count: None
                }
            ],
            label: Some("bind_group_layout")
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<ShaderConstants>() as u32
            }]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main_vs",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: node_buffer.as_entire_binding()
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: voxel_buffer.as_entire_binding()
            }]
        });
        
        let start_time = Instant::now();

        let shader_constants = ShaderConstants {
            width: size.width,
            height: size.height,
            time: start_time.elapsed().as_secs_f32()
        };


        Self {
            size,
            window,

            surface,
            device,
            queue,
            config,

            start_time,

            render_pipeline,
            shader_constants,

            // svo,
            // node_buffer,
            // voxel_buffer,

            bind_group
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.window.request_redraw();
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool { false }

    pub fn update(&mut self) {}

    pub async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.shader_constants.width = self.size.width;
        self.shader_constants.height = self.size.height;
        self.shader_constants.time = self.start_time.elapsed().as_secs_f32();

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0
                        }),
                        store: true
                    }
                })],
                depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, bytemuck::bytes_of(&self.shader_constants));
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
