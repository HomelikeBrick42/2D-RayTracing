#![deny(elided_lifetimes_in_paths)]

use anyhow::{bail, Result};
use wgpu::include_wgsl;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

#[allow(unused)]
pub struct Game {
    last_update_time: std::time::Instant,
    render_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface, // SAFETY: this needs to be before window so it gets destroyed first
    instance: wgpu::Instance,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
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
            render_pipeline,
            queue,
            device,
            adapter,
            surface_config,
            surface,
            instance,
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
                } => self.resize(new_size),
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

                let mut encoder =
                    self.device
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
                    render_pass.draw(0..4, 0..1);
                }
                self.queue.submit([encoder.finish()]);

                output.present();
            }

            Event::MainEventsCleared => {
                let time = std::time::Instant::now();
                let dt = time.duration_since(self.last_update_time);
                self.last_update_time = time;

                let dt = dt.as_secs_f64();
                self.window.set_title(&format!(
                    "FPS: {:.0}, Frame time: {:.3}ms",
                    1.0 / dt,
                    dt * 1000.0
                ));

                self.window.request_redraw();
            }

            _ => {}
        }

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }
}
