mod app;
mod viewport;
mod vertex;
mod camera;
mod mesh_operation;
mod view;

use app::App;
use eframe::epaint::vec2;

fn main() {

    let options = eframe::NativeOptions {
		renderer: eframe::Renderer::Wgpu,
		depth_buffer: 32,
		min_window_size: Some(vec2(750.0, 400.0)),
		..Default::default()
	};
    eframe::run_native("Terrain Constructor", options, Box::new(|cc| Box::new(App::new(cc).unwrap())));
}