use crate::camera::CameraUniform;


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
	pub camera: CameraUniform,
	pub display: u32, // 0 - wireframe, 1 - flat, 2 - smooth
	pub material_threshold: f32,
	pub material_smoothness: f32,
	pub _p1: u32,
}

impl ViewUniform {
	pub fn new(camera: CameraUniform, display: u32, material_threshold: f32, material_smoothness: f32) -> ViewUniform {
		Self {
			camera,
			display,
			material_smoothness,
			material_threshold,
			_p1: 0,
		}
	}
}