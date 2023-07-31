#![deny(elided_lifetimes_in_paths)]

mod vector2;
mod vector3;

pub use vector2::*;
pub use vector3::*;

use anyhow::{bail, Result};
use encase::{DynamicStorageBuffer, ShaderSize, ShaderType, UniformBuffer};
use std::collections::HashMap;
use wgpu::include_wgsl;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::ControlFlow,
    window::Window,
};

#[derive(ShaderType)]
#[repr(C)]
struct Camera {
    position: Vector2<f32>,
    player_position: Vector2<f32>,
    aspect_ratio: f32,
    vertical_view_height: f32,
}

#[derive(ShaderType)]
#[repr(C)]
struct Material {
    color: Vector3<f32>,
}

#[derive(ShaderType)]
#[repr(C)]
struct Block {
    material: u32,
}

#[derive(ShaderType)]
#[repr(C)]
struct Chunk<'a> {
    position: Vector2<f32>,
    size: Vector2<u32>,
    #[size(runtime)]
    blocks: &'a [Block],
}

#[derive(Default)]
struct MouseInfo {
    position: Vector2<f32>,
    buttons: HashMap<MouseButton, ElementState>,
}

#[allow(unused)]
pub struct Game {
    last_update_time: std::time::Instant,
    last_update_times: [f64; 20],
    materials: Vec<Material>,
    blocks: Vec<Block>,
    blocks_width: u32,
    block_storage_buffer: wgpu::Buffer,
    material_storage_start: wgpu::BufferAddress,
    block_bind_group_layout: wgpu::BindGroupLayout,
    block_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    limits: wgpu::Limits,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface, // SAFETY: this needs to be before window so it gets destroyed first
    instance: wgpu::Instance,
    key_states: HashMap<VirtualKeyCode, ElementState>,
    mouse_info: MouseInfo,
    window: Window,
}

impl Game {
    pub async fn new(window: Window) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // SAFETY: surface is declared before window in the struct so the surface will be destroyed before window
        let surface = unsafe { instance.create_surface(&window) }?;

        let Some(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        else {
            bail!("Failed to find a suitable adapter");
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // TODO: double check these limits if it doesnt run on the web
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let size = window.inner_size();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_capabilities
                .formats
                .iter()
                .find(|&format| format.is_srgb())
                .copied()
                .unwrap_or(surface_capabilities.formats[0]),
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: <Camera as ShaderSize>::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(<Camera as ShaderSize>::SHADER_SIZE),
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    camera_uniform_buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let material_storage_buffer_start = 0;
        let block_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Block Storage Buffer"),
            size: <Chunk<'_> as ShaderType>::min_size().get()
                + <&[Material] as ShaderType>::min_size().get(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let block_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Block Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let block_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Block Bind Group"),
            layout: &block_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: block_storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_storage_buffer.as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &block_bind_group_layout],
                push_constant_ranges: &[],
            });

        // TODO: we are using a render pipeline for now, maybe use compute shaders in the future?
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            last_update_time: std::time::Instant::now(),
            last_update_times: std::array::from_fn(|_| 0.0),
            materials: vec![
                Material {
                    color: [1.0, 0.0, 0.0].into(),
                },
                Material {
                    color: [0.0, 1.0, 0.0].into(),
                },
                Material {
                    color: [0.0, 0.0, 1.0].into(),
                },
            ],
            blocks: vec![
                Block { material: 0 },
                Block { material: 1 },
                Block { material: 2 },
                Block { material: u32::MAX },
            ],
            blocks_width: 2,
            block_storage_buffer,
            material_storage_start: material_storage_buffer_start,
            block_bind_group,
            block_bind_group_layout,
            camera: Camera {
                position: [0.0, 0.0].into(),
                player_position: [0.5, 0.5].into(),
                aspect_ratio: size.width as f32 / size.height as f32,
                vertical_view_height: 10.0,
            },
            camera_uniform_buffer,
            camera_bind_group,
            render_pipeline,
            queue,
            limits: device.limits(),
            device,
            adapter,
            surface_config,
            surface,
            instance,
            key_states: HashMap::new(),
            mouse_info: MouseInfo::default(),
            window,
        })
    }

    pub fn handle_event(
        &mut self,
        event: Event<'_, ()>,
        control_flow: &mut ControlFlow,
    ) -> Result<()> {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, window_id } if window_id == self.window.id() => match event
            {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(new_size)
                | WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    new_inner_size: &mut new_size,
                } => self.resize(new_size)?,

                #[allow(deprecated)]
                WindowEvent::KeyboardInput {
                    device_id: _,
                    // for ignoring the modifiers field
                    input:
                        KeyboardInput {
                            scancode: _,
                            state,
                            virtual_keycode: Some(keycode),
                            modifiers: _,
                        },
                    is_synthetic: _,
                } => {
                    self.key_states.insert(keycode, state);
                }

                // for ignoring the modifiers field
                #[allow(deprecated)]
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    let previous_position = std::mem::replace(
                        &mut self.mouse_info.position,
                        [position.x as f32, position.y as f32].into(),
                    );
                    if let Some(ElementState::Pressed) =
                        self.mouse_info.buttons.get(&MouseButton::Right)
                    {
                        let movement = self.mouse_info.position - previous_position;
                        let scale =
                            self.surface_config.height as f32 / self.camera.vertical_view_height;
                        self.camera.position.x -= movement.x / scale;
                        self.camera.position.y += movement.y / scale;
                    }
                }

                // for ignoring the modifiers field
                #[allow(deprecated)]
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    modifiers: _,
                } => {
                    self.mouse_info.buttons.insert(button, state);
                }

                // for ignoring the modifiers field
                #[allow(deprecated)]
                WindowEvent::MouseWheel {
                    device_id: _,
                    delta,
                    phase: _,
                    modifiers: _,
                } => {
                    let [_delta_x, delta_y] = match delta {
                        MouseScrollDelta::LineDelta(x, y) => [x, y],
                        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                            [x as f32, y as f32]
                        }
                    };
                    let old_scale =
                        self.surface_config.height as f32 / self.camera.vertical_view_height;
                    if delta_y > 0.0 {
                        self.camera.vertical_view_height *= 0.9 * delta_y.abs();
                    } else {
                        self.camera.vertical_view_height /= 0.9 * delta_y.abs();
                    }
                    let new_scale =
                        self.surface_config.height as f32 / self.camera.vertical_view_height;

                    let mouse_position_centered: Vector2<_> = [
                        self.mouse_info.position.x - self.surface_config.width as f32 * 0.5,
                        self.surface_config.height as f32 * 0.5 - self.mouse_info.position.y,
                    ]
                    .into();
                    self.camera.position += (mouse_position_centered
                        / [old_scale, old_scale].into())
                        - (mouse_position_centered / [new_scale, new_scale].into());
                }

                WindowEvent::Focused(false) => {
                    self.mouse_info.buttons.clear();
                    self.key_states.clear();
                }

                _ => {}
            },

            Event::RedrawRequested(window_id) if window_id == self.window.id() => 'render: {
                let output = match self.surface.get_current_texture() {
                    Ok(output) => output,
                    // should be fine next time
                    Err(wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Outdated) => {
                        break 'render;
                    }
                    // cant do much so just bail
                    Err(err) => bail!(err),
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                self.render(view)?;

                output.present();
            }

            Event::MainEventsCleared => {
                let time = std::time::Instant::now();
                let dt = time.duration_since(self.last_update_time);
                self.last_update_time = time;

                let dt = dt.as_secs_f64();
                self.last_update_times.rotate_right(1);
                self.last_update_times[0] = dt;

                {
                    let average_update_time = self.last_update_times.iter().sum::<f64>()
                        / self.last_update_times.len() as f64;
                    self.window.set_title(&format!(
                        "FPS: {:.0}, Update time: {:.3}ms",
                        1.0 / average_update_time,
                        average_update_time * 1000.0
                    ));
                }

                self.update(dt as f32)?;

                self.window.request_redraw();
            }

            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, ts: f32) -> Result<()> {
        const PLAYER_SPEED: f32 = 5.0;

        if let Some(ElementState::Pressed) = self.key_states.get(&VirtualKeyCode::W) {
            self.camera.player_position.y += PLAYER_SPEED * ts;
        }
        if let Some(ElementState::Pressed) = self.key_states.get(&VirtualKeyCode::S) {
            self.camera.player_position.y -= PLAYER_SPEED * ts;
        }
        if let Some(ElementState::Pressed) = self.key_states.get(&VirtualKeyCode::A) {
            self.camera.player_position.x -= PLAYER_SPEED * ts;
        }
        if let Some(ElementState::Pressed) = self.key_states.get(&VirtualKeyCode::D) {
            self.camera.player_position.x += PLAYER_SPEED * ts;
        }

        Ok(())
    }

    fn render(&mut self, view: wgpu::TextureView) -> Result<()> {
        // Camera uniform buffer
        {
            let mut buffer =
                UniformBuffer::new([0; <Camera as ShaderSize>::SHADER_SIZE.get() as _]);
            buffer.write(&self.camera)?;
            let buffer = buffer.into_inner();
            self.queue
                .write_buffer(&self.camera_uniform_buffer, 0, &buffer);
        }

        // Set chunk data
        {
            let mut buffer = Vec::new();

            let mut chunk_buffer: DynamicStorageBuffer<&mut Vec<u8>> =
                DynamicStorageBuffer::new(&mut buffer);
            assert!(
                self.blocks.is_empty() || (self.blocks.len() % self.blocks_width as usize == 0)
            );
            chunk_buffer.write(&Chunk {
                position: [0.0, 0.0].into(),
                size: [
                    self.blocks_width,
                    u32::try_from(self.blocks.len())?
                        .checked_div(self.blocks_width)
                        .unwrap_or(0),
                ]
                .into(),
                blocks: &self.blocks,
            })?;

            self.material_storage_start = chunk_buffer.into_inner().len().try_into()?;
            let align = self.material_storage_start
                % self.limits.min_storage_buffer_offset_alignment as wgpu::BufferAddress;
            if align != 0 {
                let grow =
                    self.limits.min_storage_buffer_offset_alignment as wgpu::BufferAddress - align;
                self.material_storage_start += grow;
                buffer.resize(self.material_storage_start as usize, 0);
            }

            let mut material_buffer = DynamicStorageBuffer::new(&mut buffer);
            material_buffer.set_offset(self.material_storage_start);
            material_buffer.write(&&self.materials)?;

            if buffer.len()
                > (self.block_storage_buffer.size() - <&[Material] as ShaderType>::min_size().get())
                    as usize
            {
                self.block_storage_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Block Storage Buffer"),
                    size: wgpu::BufferAddress::try_from(buffer.len())?
                        + <&[Material] as ShaderType>::min_size().get(),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            // TODO: only do this if the blocks have changed
            self.block_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Block Bind Group"),
                layout: &self.block_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.block_storage_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.block_storage_buffer,
                            offset: self.material_storage_start,
                            size: None,
                        }),
                    },
                ],
            });
            self.queue
                .write_buffer(&self.block_storage_buffer, 0, &buffer);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.block_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }
        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) -> Result<()> {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        self.camera.aspect_ratio =
            self.surface_config.width as f32 / self.surface_config.height as f32;

        Ok(())
    }
}
