use eframe::{egui_wgpu::WgpuConfiguration, run_native, wgpu, NativeOptions, Renderer};
use raytracing_2d::App;

fn main() {
    run_native(
        "2D Ray Tracing",
        NativeOptions {
            renderer: Renderer::Wgpu,
            vsync: false,
            wgpu_options: WgpuConfiguration {
                supported_backends: wgpu::Backends::all(),
                present_mode: wgpu::PresentMode::AutoNoVsync,
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap()
}
