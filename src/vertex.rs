use egui_wgpu::wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub pos: [f32; 4],
	pub normal: [f32; 4],
}

impl Vertex {
	pub fn new(x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32) -> Self {
		Self {
			pos: [x, y, z, 1.0],
			normal: [nx, ny, nz, 1.0],
		}
	}

	pub fn mid_ignore_normals(one: Vertex, two: Vertex) -> Vertex {
		Vertex { 
			pos: [
				one.pos[0] * 0.5 + two.pos[0] * 0.5,
				one.pos[1] * 0.5 + two.pos[1] * 0.5,
				one.pos[2] * 0.5 + two.pos[2] * 0.5,
				one.pos[3] * 0.5 + two.pos[3] * 0.5,
			], 
			normal: one.normal
		}
	}


	pub fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout { 
			array_stride: std::mem::size_of::<Vertex>() as u64, 
			step_mode: wgpu::VertexStepMode::Vertex, 
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					format: wgpu::VertexFormat::Float32x4,
					shader_location: 0,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 4]>() as u64,
					format: wgpu::VertexFormat::Float32x4,
					shader_location: 1,
				},
			], 
		}
	}
}