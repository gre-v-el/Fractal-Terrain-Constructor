struct Uniform {
	proj: mat4x4<f32>,
	eye: vec4<f32>,
	display: u32,
	material_threshold: f32,
	material_smoothness: f32,
}

@group(0) @binding(0)
var<uniform> uni: Uniform;


struct VertexIn {
	@location(0) pos: vec4<f32>,
	@location(1) normal: vec4<f32>,
}

struct VertexOut {
	@builtin(position) pos_builtin: vec4<f32>,
	@location(0) pos_world: vec4<f32>,
	@location(2) normal: vec4<f32>,
}


@vertex
fn vs_main(in: VertexIn) -> VertexOut {
	var out: VertexOut;

	out.pos_builtin = uni.proj * in.pos;
	out.pos_world = in.pos;
	out.normal = in.normal;

	return out;
}

struct Material {
	surface_color: vec3<f32>,
	ambient_color: vec3<f32>,
	ambient_strength: f32,
	diffuse_strength: f32,
	specular_strength: f32,
	roughness: f32,
}

fn material(col: vec3<f32>, amb: vec3<f32>, amb_str: f32, diff_str: f32, spec_str: f32, rough: f32) -> Material {
	var material: Material;
	material.surface_color = col;
	material.ambient_color = amb;
	material.ambient_strength = amb_str;
	material.diffuse_strength = diff_str;
	material.specular_strength = spec_str;
	material.roughness = rough;

	return material;
}

fn lerp_material(m1: Material, m2: Material, t: f32) -> Material {
	// let t = clamp(t, 0.0, 1.0);
	var m3: Material;

	m3.surface_color = 		((1.0 - t) * m1.surface_color		 + t * m2.surface_color);
	m3.ambient_color = 		((1.0 - t) * m1.ambient_color		 + t * m2.ambient_color);
	m3.ambient_strength = 	((1.0 - t) * m1.ambient_strength	 + t * m2.ambient_strength);
	m3.diffuse_strength = 	((1.0 - t) * m1.diffuse_strength	 + t * m2.diffuse_strength);
	m3.specular_strength = 	((1.0 - t) * m1.specular_strength	 + t * m2.specular_strength);
	m3.roughness = 			((1.0 - t) * m1.roughness			 + t * m2.roughness);

	return m3;
}

fn get_ambient(material: Material) -> vec3<f32> {
	return material.surface_color * material.ambient_color * material.ambient_strength;
}

fn get_diffuse(material: Material, light_color: vec3<f32>, light_dir: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
	return material.diffuse_strength * material.surface_color * light_color * max(dot(light_dir, normal), 0.0);
}

fn get_specular(material: Material, view_dir: vec3<f32>, light_dir: vec3<f32>, normal: vec3<f32>, light_color: vec3<f32>) -> vec3<f32> {
	let reflection = reflect(light_dir, normal);
	return material.specular_strength * pow(max(dot(view_dir, reflection), 0.0), 1.0/material.roughness) * light_color;
}

fn smthstp(a: f32, minimum: f32, maximum: f32) -> f32 {
	let v = min(max(a, minimum), maximum);
	let w = (v-minimum)/(maximum-minimum);

	return 3.0*w*w - 2.0*w*w*w;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
	if(uni.display == u32(0)) {
		return vec4(1.0);
	}
	// let grass = material(vec3(0.9, 0.3, 0.8), vec3(0.1, 0.5, 1.0), 0.1, 0.7, 0.2, 0.6);
	// let rock =  material(vec3(0.1, 0.3, 0.9), vec3(0.1, 0.5, 1.0), 0.1, 0.6, 0.4, 0.7);
	let grass = material(vec3(0.1, 0.7, 0.3), vec3(0.3, 0.8, 1.0), 0.6, 0.7, 0.2, 0.6);
	let rock =  material(vec3(0.2, 0.2, 0.2), vec3(0.3, 0.8, 1.0), 0.6, 0.9, 0.4, 0.7);

	let light_color = vec3(1.0, 0.9, 0.7);


	let light = vec3(-10.0, 30.0, -20.0);
	let light_dir = -normalize(light - in.pos_world.xyz);
	let view_dir = normalize(uni.eye.xyz - in.pos_world.xyz);
	var normal: vec3<f32>;
	if uni.display == u32(1) {
		normal = normalize(cross(dpdx(in.pos_world).xyz, dpdy(in.pos_world).xyz));
	}
	else {
		normal = normalize(in.normal.xyz);
		if(dot(normal, view_dir) > 0.1) {
			normal = - normal;
		}
	}

	let t = smthstp(-normal.y, uni.material_threshold - uni.material_smoothness, uni.material_threshold + uni.material_smoothness);
	// let t = smthstp(-normal.y, 0.6, 0.8);
	let material = lerp_material(rock, grass, t);


	let ambient =   get_ambient(material);
	let diffuse =   get_diffuse(material, light_color, light_dir, normal);
	let specular = get_specular(material, view_dir, light_dir, normal, light_color);

	return vec4(vec3(ambient + diffuse + specular), 1.0);
}