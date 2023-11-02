use eframe::{
    egui,
    wgpu::{self, include_wgsl},
};
use encase::{ShaderSize, ShaderType, UniformBuffer};

#[derive(ShaderType)]
struct GpuCamera {
    position: cgmath::Vector2<f32>,
    height: f32,
    player_position: cgmath::Vector2<f32>,
}

pub struct App {
    egui_texture_id: egui::TextureId,
    main_texture: wgpu::Texture,
    output_texture_bind_group_layout: wgpu::BindGroupLayout,
    output_texture_bind_group: wgpu::BindGroup,
    camera: GpuCamera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    camera_window: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let eframe::egui_wgpu::RenderState {
            device, renderer, ..
        } = cc.wgpu_render_state.as_ref().unwrap();

        let main_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let output_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Output Texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: main_texture.format(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let output_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Output Texture Bind Group"),
            layout: &output_texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &main_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: <GpuCamera as ShaderSize>::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(<GpuCamera as ShaderSize>::SHADER_SIZE),
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&output_texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            egui_texture_id: renderer.write().register_native_texture(
                device,
                &main_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                wgpu::FilterMode::Nearest,
            ),
            main_texture,
            output_texture_bind_group_layout,
            output_texture_bind_group,
            camera: GpuCamera {
                position: cgmath::vec2(0.0, 0.0),
                height: 1.0,
                player_position: cgmath::vec2(0.0, 0.0),
            },
            camera_buffer,
            camera_bind_group,
            compute_pipeline,
            camera_window: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Top Panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.camera_window |= ui.button("Camera").clicked();
            });
        });

        egui::Window::new("Camera")
            .open(&mut self.camera_window)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position: ");
                    ui.add(
                        egui::DragValue::new(&mut self.camera.position.x)
                            .prefix("x:")
                            .speed(0.01),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.camera.position.y)
                            .prefix("y:")
                            .speed(0.01),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("View Height: ");
                    ui.add(egui::DragValue::new(&mut self.camera.height).speed(0.1));
                    self.camera.height = self.camera.height.max(0.01);
                });

                ui.horizontal(|ui| {
                    ui.label("Player Position: ");
                    ui.add(
                        egui::DragValue::new(&mut self.camera.player_position.x)
                            .prefix("x:")
                            .speed(0.01),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.camera.player_position.y)
                            .prefix("y:")
                            .speed(0.01),
                    );
                });

                ui.allocate_space(ui.available_size());
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(255, 0, 255)))
            .show(ctx, |ui| {
                let eframe::egui_wgpu::RenderState {
                    device,
                    queue,
                    renderer,
                    ..
                } = frame.wgpu_render_state().unwrap();

                let (_, rect) = ui.allocate_space(ui.available_size());

                // Resize output texture if needed
                let (width, height) = (rect.width() as i64, rect.height() as i64);
                if self.main_texture.width() as i64 != width
                    && self.main_texture.height() as i64 != height
                    && width > 0
                    && height > 0
                {
                    self.main_texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Main Texture"),
                        size: wgpu::Extent3d {
                            width: width as _,
                            height: height as _,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::STORAGE_BINDING,
                        view_formats: &[],
                    });
                    renderer.write().update_egui_texture_from_wgpu_texture(
                        device,
                        &self
                            .main_texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        wgpu::FilterMode::Nearest,
                        self.egui_texture_id,
                    );
                    self.output_texture_bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Output Texture Bind Group"),
                            layout: &self.output_texture_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &self
                                        .main_texture
                                        .create_view(&wgpu::TextureViewDescriptor::default()),
                                ),
                            }],
                        });
                }

                // Upload camera uniform
                {
                    let mut buffer =
                        UniformBuffer::new([0; <GpuCamera as ShaderSize>::SHADER_SIZE.get() as _]);
                    buffer.write(&self.camera).unwrap();
                    let buffer = buffer.into_inner();
                    queue.write_buffer(&self.camera_buffer, 0, &buffer);
                }

                let mut command_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Compute Command Encoder"),
                    });
                {
                    let mut compute_pass =
                        command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Compute Pass"),
                        });

                    let (workgroup_width, workgroup_height) = (16, 16);
                    let (workgroups_x, workgroups_y) = (
                        (self.main_texture.width() + workgroup_width - 1) / workgroup_width,
                        (self.main_texture.height() + workgroup_height - 1) / workgroup_height,
                    );

                    compute_pass.set_pipeline(&self.compute_pipeline);
                    compute_pass.set_bind_group(0, &self.output_texture_bind_group, &[]);
                    compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
                }
                queue.submit([command_encoder.finish()]);

                ui.painter().image(
                    self.egui_texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });
    }
}
