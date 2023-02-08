use std::{ops::RangeInclusive, collections::{HashMap, HashSet}};

use cgmath::Vector4;
use eframe::egui::{self, Ui, WidgetText};
use rand::{rngs::StdRng, RngCore};

use crate::vertex::Vertex;

fn v_label_drag(ui: &mut Ui, label: Option<impl Into<WidgetText>>, n: &mut impl egui::emath::Numeric, range: RangeInclusive<impl egui::emath::Numeric>, speed: f32) {
	ui.vertical(|ui| {
		if let Some(label) = label {
			ui.label(label);
		}
		ui.add(egui::DragValue::new(n).clamp_range(range).speed(speed));
	});
}

// todo: Erosion, Unsubdivide, Fractal

pub enum ShowResponse {
	None,
	Delete,
	MoveUp,
	MoveDown,
	GenerateUpToThis,
}

// The two variants beginning with '_':
// Subdivide smooth - not implemented in the end, because a simmilar effect can be achieved by first subdividing, then smoothing
// MergeCleanup - it is no longer needed, since Sybdivide has been rewritten and no longer needs a cleanup afterwards
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MeshOperation {
	AddTriangle(f32),							// size
	AddTriSquare(f32),							// size
	AddTriangleGrid(f32, u32),					// size, subdivisions
	AddTriSquareGrid(f32, u32),					// size, subdivisions
	
	Subdivide(u32),								// subdivisions
	_SubdivideSmooth(u32, f32),					// subdivisions, smoothness
	
	DisplaceRandom(f32, [bool; 3]),				// amount, axes
	DisplaceSmooth(f32, f32, u32, [bool; 3]),	// amount, scale, octaves, axes
	Smooth(f32, u32),							// amount, iterations

	_MergeCleanup,

	FractalTerrain(u32, f32, f32),				// iterations, displacement start, displacement decay
}

impl MeshOperation {
	pub const DEFAULTS: [MeshOperation; 9] = [
		MeshOperation::AddTriangle(5.0),
		MeshOperation::AddTriSquare(5.0),
		MeshOperation::AddTriangleGrid(10.0, 20),
		MeshOperation::AddTriSquareGrid(10.0, 20),
		MeshOperation::Subdivide(1),
		// MeshOperation::SubdivideSmooth(5, 1.0),
		MeshOperation::DisplaceRandom(0.2, [false, true, false]),
		MeshOperation::DisplaceSmooth(1.0, 1.0, 1, [false, true, false]),
		MeshOperation::Smooth(0.5, 1),
		// MeshOperation::MergeCleanup,
		MeshOperation::FractalTerrain(6, 2.0, 2.0),
	];
	
	pub fn show(&mut self, ui: &mut egui::Ui, time: f32) -> ShowResponse {
		let mut ret = ShowResponse::None;
		
		ui.vertical(|ui| {
			
			ui.heading(self.caption());
			
			match self {
				Self::AddTriangle(r) => {
					v_label_drag(ui, Some("size:"), r, 0.0..=f32::MAX, 0.01);
				},
				Self::AddTriSquare(r) => {
					v_label_drag(ui, Some("size:"), r, 0.0..=f32::MAX, 0.01);
				},
				Self::AddTriangleGrid(r, n) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("size:"), r, 0.0..=f32::MAX, 0.01);
						v_label_drag(ui, Some("subdivisions:"), n, 0..=u32::MAX, 1.0);
					});
				},
				Self::AddTriSquareGrid(r, n) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("size:"), r, 0.0..=f32::MAX, 0.01);
						v_label_drag(ui, Some("subdivisions:"), n, 0..=u32::MAX, 1.0);
					});
				},
				
				Self::Subdivide(n) => {
					v_label_drag(ui, Some("subdivisions:"), n, 0..=u32::MAX, 1.0);
				},
				Self::_SubdivideSmooth(n, smoothness) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("subdivisions:"), n, 0..=u32::MAX, 1.0);
						v_label_drag(ui, Some("smoothness"), smoothness, 0.0..=1.0, 0.01);
					});
				},
				
				Self::DisplaceRandom(r, axes) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("amount:"), r, 0.0..=f32::MAX, 0.01);

						ui.vertical(|ui| {
							ui.label("axes:");
							ui.checkbox(&mut axes[0], "X");
							ui.checkbox(&mut axes[1], "Y");
							ui.checkbox(&mut axes[2], "Z");
						});
					});
				},
				Self::DisplaceSmooth(r, scale, octaves, axes) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("amount:"), r, 0.0..=f32::MAX, 0.01);
						v_label_drag(ui, Some("scale:"), scale, 0.0..=f32::MAX, 0.01);
						v_label_drag(ui, Some("octaves:"), octaves, 0..=u32::MAX, 0.01);

						ui.vertical(|ui| {
							ui.label("axes:");
							ui.checkbox(&mut axes[0], "X");
							ui.checkbox(&mut axes[1], "Y");
							ui.checkbox(&mut axes[2], "Z");
						});
					});
				},
				Self::Smooth(amount, iterations) => {
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("amount:"), amount, 0.0..=1.0, 0.01);
						v_label_drag(ui, Some("iterations"), iterations, 0..=u32::MAX, 0.1);
					});
				},
				Self::_MergeCleanup => {}
				Self::FractalTerrain(iterations, displacement_start, displacement_decay) => {
					v_label_drag(ui, Some("iterations"), iterations, 1..=u32::MAX, 0.2);
					ui.horizontal(|ui| {
						v_label_drag(ui, Some("displacement start"), displacement_start, 0.0..=f32::MAX, 0.01);
						v_label_drag(ui, Some("displacement decay"), displacement_decay, 1.0..=f32::MAX, 0.01);
					});
				}
			}

			ui.label(if time > 0.0 { format!("{:.2}s", time) } else { "".to_string() });

			ui.horizontal(|ui| {
				if ui.button("delete").clicked() { ret = ShowResponse::Delete }
				if ui.button("up").clicked() { ret = ShowResponse::MoveUp }
			});
			ui.horizontal(|ui| {
				if ui.button("generate").clicked() { ret = ShowResponse::GenerateUpToThis }
				if ui.button("down").clicked() { ret = ShowResponse::MoveDown }
			})
		});

		ret
	}
	
	pub fn caption(&self) -> &str{
		match self {
			Self::AddTriangle(_) => "Add Triangle",
			Self::AddTriSquare(_) => "Add Triangle Square",
			Self::AddTriangleGrid(_, _) => "Add Triangle Grid",
			Self::AddTriSquareGrid(_, _) => "Add Triangle Square Grid",
			
			Self::Subdivide(_) => "Subdivide",
			Self::_SubdivideSmooth(_, _) => "Subdivide Smooth",
			
			Self::DisplaceRandom(_, _) => "Displace Random",
			Self::DisplaceSmooth(_, _, _, _) => "Displace Smooth",
			Self::Smooth(_, _) => "Smooth",

			Self::_MergeCleanup => "Merge Cleanup",

			Self::FractalTerrain(_, _, _) => "Fractal Terrain",
		}
	}

	pub fn execute(&self, verts_in: Vec<Vertex>, mut inds_in: Vec<u32>, random: &mut StdRng) -> (Vec<Vertex>, Vec<u32>) {
		let mut verts_out = Vec::<Vertex>::new();
		let mut inds_out = Vec::<u32>::new();

		match self {
			
			Self::AddTriangle(size) => {
				verts_out.push(Vertex::new(-0.5*size, 0.0, -3f32.sqrt()/6.0*size, 0.0, 0.0, 0.0));
				verts_out.push(Vertex::new(0.5*size, 0.0, -3f32.sqrt()/6.0*size, 0.0, 0.0, 0.0));
				verts_out.push(Vertex::new(0.0, 0.0, 3f32.sqrt()/3.0*size, 0.0, 0.0, 0.0));

				inds_out.push(0);
				inds_out.push(1);
				inds_out.push(2);
			},

			Self::AddTriSquare(size) => {
				verts_out.push(Vertex::new(-0.5*size, 0.0, -0.5*size, 0.0, 0.0, 0.0));
				verts_out.push(Vertex::new( 0.5*size, 0.0, -0.5*size, 0.0, 0.0, 0.0));
				verts_out.push(Vertex::new( 0.5*size, 0.0,  0.5*size, 0.0, 0.0, 0.0));
				verts_out.push(Vertex::new(-0.5*size, 0.0,  0.5*size, 0.0, 0.0, 0.0));

				inds_out.push(1);
				inds_out.push(2);
				inds_out.push(3);

				inds_out.push(1);
				inds_out.push(3);
				inds_out.push(0);
			},

			Self::AddTriangleGrid(size, subdivisions) => {
				let num = subdivisions + 1;
				for x in 0..num {
					let x_world = (x as f32 / *subdivisions as f32 - 0.5) * size * 3f32.sqrt() / 2.0;
			
					for z in 0..num {
						let shift = 
							if x % 2 == 0 {
								0.5
							} else {
								0.0
							};
						let z_world = ((z as f32 - shift) / *subdivisions as f32 - 0.5) * size;
							
			
						verts_out.push(Vertex::new(x_world, 0.0, z_world, 0.0, 0.0, 0.0));
			
						if x != 0 && z != 0 {
							let index = x * num + z;
			
							if x % 2 == 0 {
								inds_out.push(index);
								inds_out.push(index - num - 1);
								inds_out.push(index - 1);
				
								inds_out.push(index);
								inds_out.push(index - num);
								inds_out.push(index - num - 1);
							}
							else {
								inds_out.push(index);
								inds_out.push(index - num);
								inds_out.push(index - 1);
				
								inds_out.push(index - num);
								inds_out.push(index - num - 1);
								inds_out.push(index - 1);
							}
						}
					}
				}
			},

			Self::AddTriSquareGrid(size, subdivisions) => {
				let num = subdivisions + 1;
				for x in 0..num {
					let x_world = (x as f32 / num as f32 - 0.5) * size;
			
					for z in 0..num {
						let z_world = (z as f32 / num as f32 - 0.5) * size;
			
						verts_out.push(Vertex::new(x_world, 0.0, z_world, 0.0, 0.0, 0.0));
			
						if x != 0 && z != 0 {
							let index = x * num + z;
			
							inds_out.push(index as u32);
							inds_out.push((index - num - 1) as u32);
							inds_out.push((index - 1) as u32);
			
							inds_out.push(index as u32);
							inds_out.push((index - num) as u32);
							inds_out.push((index - num - 1) as u32);
						}
					}
				}
			},

			Self::DisplaceRandom(amount, axes) => {
				verts_out = verts_in;
				inds_out = inds_in;
				for v in verts_out.iter_mut() {
					for i in 0..axes.len() {	
						if axes[i] {
							v.pos[i] += (random.next_u32() as f32 / u32::MAX as f32) * amount;
						}
					}
				}
			},

			Self::DisplaceSmooth(amount, scale, octaves, axes) => {
				verts_out = verts_in;
				inds_out = inds_in;

				use noise::{NoiseFn, Simplex};

				let noise: Vec<_> = (0..*octaves).into_iter().map(|_| Simplex::new(random.next_u32())).collect();
				
				for v in verts_out.iter_mut() {
					for i in 0..axes.len() {	
						if axes[i] {
							let pos = [
								(v.pos[0] / scale) as f64, 
								(v.pos[1] / scale) as f64, 
								(v.pos[2] / scale) as f64
							];
							let mut sum = 0.0;
							let mut cumulator = 0.0;
							for o in 0..*octaves {
								sum += 0.5f32.powi(o as i32);
								cumulator += 0.5f32.powi(o as i32) * noise[o as usize].get(
									[pos[0] * 2f64.powi(o as i32), 
									 pos[1] * 2f64.powi(o as i32), 
									 pos[2] * 2f64.powi(o as i32)]) as f32;
							}
							cumulator /= sum;
							
							v.pos[i] += cumulator * amount;
						}
					}
				}
			}

			Self::Subdivide(iterations) => {

				verts_out = verts_in;
				
				for _ in 0..*iterations {

					let mut mids = HashMap::<(u32, u32), u32>::new();

					for i in 0..(inds_in.len()/3) {
						//       1
						//     /   \
						//    4-----3
						//  /  \   /  \
						// 2-----5-----0

						let mut inds = [
							inds_in[3*i+0], 
							inds_in[3*i+1], 
							inds_in[3*i+2], 
							0,
							0,
							0,
						];
						const IDS: [[usize; 2]; 3] = [
							[0, 1],
							[1, 2],
							[2, 0],
						];
						let found = [
							mids.get(&(inds[IDS[0][0]].min(inds[IDS[0][1]]), (inds[IDS[0][0]].max(inds[IDS[0][1]])))).is_some(),
							mids.get(&(inds[IDS[1][0]].min(inds[IDS[1][1]]), (inds[IDS[1][0]].max(inds[IDS[1][1]])))).is_some(),
							mids.get(&(inds[IDS[2][0]].min(inds[IDS[2][1]]), (inds[IDS[2][0]].max(inds[IDS[2][1]])))).is_some(),
						];



						for i in 0..3 {
							if found[i] {
								inds[3+i] = *mids.get(&(inds[IDS[i][0]].min(inds[IDS[i][1]]), (inds[IDS[i][0]].max(inds[IDS[i][1]])))).unwrap();
							}
							else {
								mids.insert((inds[IDS[i][0]].min(inds[IDS[i][1]]), (inds[IDS[i][0]].max(inds[IDS[i][1]]))), verts_out.len() as u32);
								inds[3+i] = verts_out.len() as u32;
								verts_out.push(Vertex::mid_ignore_normals(verts_out[inds[IDS[i][0]] as usize], verts_out[inds[IDS[i][1]] as usize]));
							}
						}

						inds_out.push(inds[0]);
						inds_out.push(inds[3]);
						inds_out.push(inds[5]);
						
						inds_out.push(inds[3]);
						inds_out.push(inds[1]);
						inds_out.push(inds[4]);
						
						inds_out.push(inds[5]);
						inds_out.push(inds[3]);
						inds_out.push(inds[4]);

						inds_out.push(inds[5]);
						inds_out.push(inds[4]);
						inds_out.push(inds[2]);
					}

					inds_in = inds_out;
					inds_out = Vec::new();
				}
				inds_out = inds_in;
			}

			Self::_MergeCleanup => {
				inds_out = inds_in;

				let mut verts = verts_in.as_slice();

				/*
					for each vertex, check if there are any duplicates
					if yes, don't keep it
					if no, keep it

					if it's not added, check if any indices pointed to the vertex, and point them to the other one
					decrease all other indices above the vertex's index by one
				*/
				
				let mut dropped = 0;
				let mut first_index_original = 0;
				while let Some((first, rest)) = verts.split_first() {
					let mut matched = None;
					let mut second_index_original = first_index_original + 1;
					for second in rest.iter() {
						if second.pos == first.pos {
							matched = Some(second_index_original);
							break;
						}
						second_index_original += 1;
					}

					if let Some(second_index) = matched {
						for i in 0..inds_out.len() {
							if inds_out[i] == first_index_original - dropped {
								inds_out[i] = second_index - dropped;
							}
						}
						for i in 0..inds_out.len() {
							if inds_out[i] > first_index_original - dropped {
								inds_out[i] -= 1;
							}
						}
						dropped += 1;
					}
					else {
						verts_out.push(*first);
					}

					verts = rest;
					first_index_original += 1;
				}

			}

			Self::Smooth(amount, iterations) => {
				
				inds_out = inds_in;
				verts_out = verts_in;
				for _ in 0..*iterations {
					let mut verts: Vec<(Vertex, Vector4<f32>)> = verts_out.iter().map(|v| (*v, [0.0, 0.0, 0.0, 0.0].into())).collect();
					
					for i in 0..inds_out.len()/3 {
						let i0 = inds_out[3*i+0] as usize;
						let i1 = inds_out[3*i+1] as usize;
						let i2 = inds_out[3*i+2] as usize;

						let v0 = verts[i0].0;
						let v1 = verts[i1].0;
						let v2 = verts[i2].0;

						verts[i0].1 += Vector4::from(v1.pos);
						verts[i0].1 += Vector4::from(v2.pos);

						verts[i1].1 += Vector4::from(v0.pos);
						verts[i1].1 += Vector4::from(v2.pos);

						verts[i2].1 += Vector4::from(v0.pos);
						verts[i2].1 += Vector4::from(v1.pos);
					}

					for v in verts.iter_mut() {
						let mut pos = Vector4::from(v.0.pos).truncate();
						let target = v.1.truncate() / v.1.w;

						pos.x = amount * target.x + (1.0 - amount) * pos.x;
						pos.y = amount * target.y + (1.0 - amount) * pos.y;
						pos.z = amount * target.z + (1.0 - amount) * pos.z;

						v.0.pos = pos.extend(1.0).into();
					}

					verts_out = verts.iter().map(|v| v.0).collect();
				}
			}

			Self::FractalTerrain(iterations, displacement_start, displacement_decay) => {
				// exactly like subdivide, but with vertical displacement
				
				verts_out = verts_in;
				
				for iteration in 0..*iterations {

					let mut mids = HashMap::<(u32, u32), u32>::new();

					for i in 0..(inds_in.len()/3) {
						//       1
						//     /   \
						//    4-----3
						//  /  \   /  \
						// 2-----5-----0

						let mut inds = [
							inds_in[3*i+0], 
							inds_in[3*i+1], 
							inds_in[3*i+2], 
							0,
							0,
							0,
						];
						const IDS: [[usize; 2]; 3] = [
							[0, 1],
							[1, 2],
							[2, 0],
						];
						let found = [
							mids.get(&(inds[IDS[0][0]].min(inds[IDS[0][1]]), (inds[IDS[0][0]].max(inds[IDS[0][1]])))).is_some(),
							mids.get(&(inds[IDS[1][0]].min(inds[IDS[1][1]]), (inds[IDS[1][0]].max(inds[IDS[1][1]])))).is_some(),
							mids.get(&(inds[IDS[2][0]].min(inds[IDS[2][1]]), (inds[IDS[2][0]].max(inds[IDS[2][1]])))).is_some(),
						];



						for i in 0..3 {
							if found[i] {
								inds[3+i] = *mids.get(&(inds[IDS[i][0]].min(inds[IDS[i][1]]), (inds[IDS[i][0]].max(inds[IDS[i][1]])))).unwrap();
							}
							else {
								mids.insert((inds[IDS[i][0]].min(inds[IDS[i][1]]), (inds[IDS[i][0]].max(inds[IDS[i][1]]))), verts_out.len() as u32);
								inds[3+i] = verts_out.len() as u32;

								// this part differs from the subdivide operation

								let v1 = verts_out[inds[IDS[i][0]] as usize];
								let v2 = verts_out[inds[IDS[i][1]] as usize];
								let mut mid = Vertex::mid_ignore_normals(v1, v2);

								let disp = displacement_start * displacement_decay.powf(-(iteration as f32));

								mid.pos[1] += (random.next_u32() as f32 / u32::MAX as f32 - 0.5) * disp;

								verts_out.push(mid);
							}
						}

						inds_out.push(inds[0]);
						inds_out.push(inds[3]);
						inds_out.push(inds[5]);
						
						inds_out.push(inds[3]);
						inds_out.push(inds[1]);
						inds_out.push(inds[4]);
						
						inds_out.push(inds[5]);
						inds_out.push(inds[3]);
						inds_out.push(inds[4]);

						inds_out.push(inds[5]);
						inds_out.push(inds[4]);
						inds_out.push(inds[2]);
					}

					inds_in = inds_out;
					inds_out = Vec::new();
				}
				inds_out = inds_in;
			}

			_ => {
				verts_out = verts_in;
				inds_out = inds_in;
				println!("todo");
			}
		}



		(verts_out, inds_out)
	}
}

pub fn wireframe_indices(indices: &[u32]) -> Vec<u32> {
	let mut set = HashSet::<(u32, u32)>::new();
	let mut ret = Vec::<u32>::new();

	for i in 0..(indices.len() as u32/3 as u32) {
		let (i0, i1, i2) = (indices[(3*i+0) as usize], indices[(3*i+1) as usize], indices[(3*i+2) as usize]);

		if !set.contains(&(i0.min(i1), i0.max(i1))) {
			set.insert((i0.min(i1), i0.max(i1)));
			ret.push(i0);
			ret.push(i1);
		}
		if !set.contains(&(i1.min(i2), i1.max(i2))) {
			set.insert((i1.min(i2), i1.max(i2)));
			ret.push(i1);
			ret.push(i2);
		}
		if !set.contains(&(i0.min(i2), i0.max(i2))) {
			set.insert((i0.min(i2), i0.max(i2)));
			ret.push(i0);
			ret.push(i2);
		}
	}

	return ret;
}