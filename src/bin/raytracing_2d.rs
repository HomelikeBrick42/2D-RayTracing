use std::sync::Arc;

use raytracing_2d::state::State;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct App {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    state: Option<State>,
    window_state: Option<WindowState>,
    last_frame_time: Option<std::time::Instant>,
    delta_time: std::time::Duration,
    cursor_position: PhysicalPosition<f64>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_state = None;

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("2D Ray Tracer"))
                .expect("window should be successfully created"),
        );

        let size = window.inner_size();

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface should be created successfully");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&self.device, &surface_config);

        self.window_state = Some(WindowState {
            window,
            surface,
            surface_config,
        });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_state = None;
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        let time = std::time::Instant::now();
        self.delta_time = time.duration_since(self.last_frame_time.unwrap_or(time));
        self.last_frame_time = Some(time);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let state = self
            .state
            .as_mut()
            .expect("the state should exist unless the app is exiting");
        state.update(self.delta_time);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window_state) = self.window_state.take() {
            window_state.window.set_visible(false);
        }
        self.state = None;
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let state = self
            .state
            .as_mut()
            .expect("the state should exist unless the app is exiting");

        if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
            state.mouse_moved(cgmath::vec2(x as f32, y as f32));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let WindowState {
            window,
            surface,
            surface_config,
            ..
        } = self
            .window_state
            .as_mut()
            .expect("if there is a window event the window should have been created");
        assert_eq!(window.id(), id);

        let state = self
            .state
            .as_mut()
            .expect("the state should exist unless the app is exiting");

        let mut resized = |surface_config: &mut wgpu::SurfaceConfiguration,
                           size: winit::dpi::PhysicalSize<u32>| {
            surface_config.width = size.width.max(1);
            surface_config.height = size.height.max(1);
            surface.configure(&self.device, surface_config);
            state.resize(&self.device, surface_config.width, surface_config.height);
        };

        match event {
            WindowEvent::Destroyed | WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => resized(surface_config, size),

            WindowEvent::RedrawRequested => {
                let surface_texture = loop {
                    match surface.get_current_texture() {
                        Ok(texture) => break texture,

                        Err(e @ wgpu::SurfaceError::Timeout) => {
                            eprintln!("WARNING: {e}");
                        }

                        Err(wgpu::SurfaceError::Outdated) => {
                            let size = window.inner_size();
                            resized(surface_config, size);
                        }

                        Err(wgpu::SurfaceError::Lost) => {
                            surface.configure(&self.device, surface_config);
                        }

                        Err(e @ (wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other)) => {
                            eprintln!("ERROR: {e}");
                            return;
                        }
                    }
                };

                state.render(&self.device, &self.queue, &surface_texture.texture);

                window.pre_present_notify();
                surface_texture.present();
                window.request_redraw();
            }

            WindowEvent::Focused(focused) => {
                state.focused(focused, window);
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: key_state,
                        ..
                    },
                is_synthetic: _,
            } => state.key(key, key_state, window),

            WindowEvent::MouseWheel {
                device_id: _,
                delta: MouseScrollDelta::LineDelta(x, y),
                phase: _,
            } => state.mouse_scrolled(cgmath::vec2(x, y)),

            WindowEvent::MouseWheel {
                device_id: _,
                delta: MouseScrollDelta::PixelDelta(delta),
                phase: _,
            } => state.mouse_scrolled(cgmath::vec2(delta.x as f32, delta.y as f32)),

            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_position = position;
                let PhysicalSize { width, height } = window.inner_size();
                state.cursor_moved(cgmath::Vector2 {
                    x: (((self.cursor_position.x as f32 + 0.5) / width as f32) * 2.0 - 1.0)
                        * (width as f32 / height as f32),
                    y: ((self.cursor_position.y as f32 + 0.5) / height as f32) * -2.0 + 1.0,
                });
            }

            WindowEvent::MouseInput {
                device_id: _,
                state: button_state,
                button,
            } => {
                let PhysicalSize { width, height } = window.inner_size();
                state.mouse(
                    button,
                    button_state,
                    cgmath::Vector2 {
                        x: (((self.cursor_position.x as f32 + 0.5) / width as f32) * 2.0 - 1.0)
                            * (width as f32 / height as f32),
                        y: ((self.cursor_position.y as f32 + 0.5) / height as f32) * -2.0 + 1.0,
                    },
                );
            }

            _ => {}
        }
    }
}

fn main() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("an adapter should have been requested successfully");

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("device should have been requested successfully");

    let state = State::new(&device, &queue);

    let event_loop = EventLoop::new().expect("the event loop should be created");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut App {
            instance,
            device,
            queue,
            state: Some(state),
            window_state: None,
            last_frame_time: None,
            delta_time: std::time::Duration::ZERO,
            cursor_position: PhysicalPosition { x: 0.0, y: 0.0 },
        })
        .expect("the event loop should be started");
}
