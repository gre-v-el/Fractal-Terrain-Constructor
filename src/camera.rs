use cgmath::{Rotation, EuclideanSpace};
use eframe::egui;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	pub mat: [[f32; 4]; 4],
	pub eye: [f32; 4]
}

pub struct OrbitCamera {
	pub origin: cgmath::Point3<f32>,
	pub radius: f32,
	pub y_rot: f32,
	pub x_rot: f32,
	pub aspect: f32,
}

impl OrbitCamera {
	pub fn get_uniform(&self) -> CameraUniform {

		let x_rot = cgmath::Quaternion::from(
			cgmath::Euler {
				x: cgmath::Deg(self.x_rot),
				y: cgmath::Deg(0.0),
				z: cgmath::Deg(0.0),
			}
		);
		let y_rot = cgmath::Quaternion::from(
			cgmath::Euler {
				x: cgmath::Deg(0.0),
				y: cgmath::Deg(self.y_rot),
				z: cgmath::Deg(0.0),
			}
		);

		let eye = self.origin - y_rot.rotate_vector(x_rot.rotate_vector(cgmath::Vector3::unit_z() * self.radius));
		let view = cgmath::Matrix4::look_at_rh(eye, self.origin, cgmath::Vector3::unit_y());
		let projection = cgmath::perspective(cgmath::Deg(60.0) , self.aspect, 0.1, 100.0);

		CameraUniform { 
			mat: (projection * view).into(),
			eye: eye.to_vec().extend(1.0).into(),
		}
	}

	pub fn update(&mut self, response: &egui::Response, scroll: egui::emath::Vec2) {
		self.x_rot += response.drag_delta().y * 0.5;
		self.y_rot -= response.drag_delta().x * 0.5;

		if response.hovered() {
			self.radius /= 1.001f32.powf(scroll.y);
		}
	}
}