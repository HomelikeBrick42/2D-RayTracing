use crate::gpu_buffers::{BufferCreationInfo, BufferGroup, FixedSizeBuffer};
use encase::ShaderType;
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

struct Camera {
    position: cgmath::Vector2<f32>,
    height: f32,
}

#[derive(ShaderType)]
struct GpuCamera {
    position: cgmath::Vector2<f32>,
    height: f32,
    aspect: f32,
}

impl GpuCamera {
    fn from_camera(camera: &Camera, aspect: f32) -> Self {
        let Camera { position, height } = *camera;
        Self {
            position,
            height,
            aspect,
        }
    }
}

pub struct State {
    camera: Camera,
    camera_buffer: BufferGroup<(FixedSizeBuffer<GpuCamera>,)>,

    ray_tracing_render_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> State {
        let camera = Camera {
            position: cgmath::vec2(0.0, 0.0),
            height: 1.0,
        };
        let camera_buffer = BufferGroup::new(
            device,
            "Camera Bind Group",
            (BufferCreationInfo {
                buffer: FixedSizeBuffer::new(
                    device,
                    queue,
                    "Camera Buffer",
                    wgpu::BufferUsages::UNIFORM,
                    &GpuCamera::from_camera(&camera, 1.0),
                ),
                binding_type: wgpu::BufferBindingType::Uniform,
                visibility: wgpu::ShaderStages::FRAGMENT,
            },),
        );

        let ray_tracing_shader = unsafe {
            device.create_shader_module_passthrough(wgpu::include_spirv_raw!(concat!(
                env!("OUT_DIR"),
                "/shaders/ray_tracing.spv"
            )))
        };
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Render Pipeline Layout"),
                bind_group_layouts: &[camera_buffer.bind_group_layout()],
                push_constant_ranges: &[],
            });
        let ray_tracing_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Ray Tracing Render Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                vertex: wgpu::VertexState {
                    module: &ray_tracing_shader,
                    entry_point: Some("vertex"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &ray_tracing_shader,
                    entry_point: Some("fragment"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        State {
            camera,
            camera_buffer,

            ray_tracing_render_pipeline,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        let ts = dt.as_secs_f32();
        _ = ts;
    }

    pub fn key(&mut self, key: KeyCode, state: ElementState, window: &winit::window::Window) {
        _ = key;
        _ = state;
        _ = window;
    }

    pub fn mouse(&mut self, button: MouseButton, state: ElementState, uv: cgmath::Vector2<f32>) {
        _ = button;
        _ = state;
        _ = uv;
    }

    pub fn focused(&mut self, focused: bool, window: &winit::window::Window) {
        _ = focused;
        _ = window;
    }

    pub fn mouse_scrolled(&mut self, delta: cgmath::Vector2<f32>) {
        _ = delta;
    }

    pub fn mouse_moved(&mut self, delta: cgmath::Vector2<f32>) {
        _ = delta;
    }

    pub fn cursor_moved(&mut self, uv: cgmath::Vector2<f32>) {
        _ = uv;
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        _ = device;
        _ = width;
        _ = height;
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let wgpu::Extent3d { width, height, .. } = texture.size();

        self.camera_buffer.write(
            device,
            queue,
            (Some(&GpuCamera::from_camera(
                &self.camera,
                width as f32 / height as f32,
            )),),
        );

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Rendering Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&Default::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(&self.ray_tracing_render_pipeline);
            render_pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);
            render_pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(command_encoder.finish()));
    }
}
