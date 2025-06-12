#![no_std]

use spirv_std::glam::{vec2, vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::image::SampledImage;
use spirv_std::{num_traits::Float, spirv, Image};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec4,
    pub color_radius: Vec4, // color in xyz, radius in w
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub lights: [Light; 6],
    pub view_pos: Vec4,
    pub display_debug_target: i32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_index: i32,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    let uv = vec2(((vert_index << 1) & 2) as f32, (vert_index & 2) as f32);
    *out_uv = uv;
    *out_position = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_position: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_normal: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(descriptor_set = 0, binding = 3)] sampler_albedo: &SampledImage<
        Image!(2D, type=f32, sampled),
    >,
    #[spirv(uniform, descriptor_set = 0, binding = 4)] ubo: &UBO,
    out_frag_color: &mut Vec4,
) {
    // Get G-Buffer values
    let frag_pos = sampler_position.sample(in_uv).xyz();
    let normal = sampler_normal.sample(in_uv).xyz();
    let albedo = sampler_albedo.sample(in_uv);

    // Use the full lighting calculation to process all 6 lights

    // Debug display
    if ubo.display_debug_target > 0 {
        match ubo.display_debug_target {
            1 => *out_frag_color = vec4(frag_pos.x, frag_pos.y, frag_pos.z, 1.0),
            2 => *out_frag_color = vec4(normal.x, normal.y, normal.z, 1.0),
            3 => *out_frag_color = vec4(albedo.x, albedo.y, albedo.z, 1.0),
            4 => *out_frag_color = vec4(albedo.w, albedo.w, albedo.w, 1.0),
            _ => {}
        }
        return;
    }

    // Render-target composition
    const LIGHT_COUNT: usize = 6;
    const AMBIENT: f32 = 0.0;

    // Ambient part
    let mut frag_color = albedo.xyz() * AMBIENT;

    for i in 0..LIGHT_COUNT {
        // Vector to light
        let l_vec = ubo.lights[i].position.xyz() - frag_pos;
        // Distance from light to fragment position
        let dist = l_vec.length();

        // Viewer to fragment
        let v = (ubo.view_pos.xyz() - frag_pos).normalize();

        // Light to fragment
        let l = l_vec.normalize();

        // Attenuation
        let atten = ubo.lights[i].color_radius.w / (dist.powf(2.0) + 1.0);

        // Diffuse part
        let n = normal.normalize();
        let n_dot_l = n.dot(l).max(0.0);
        let diff = ubo.lights[i].color_radius.xyz() * albedo.xyz() * n_dot_l * atten;

        // Specular part
        // Specular map values are stored in alpha of albedo mrt
        let r = (-l).reflect(n);
        let n_dot_r = r.dot(v).max(0.0);
        let spec = ubo.lights[i].color_radius.xyz() * albedo.w * n_dot_r.powf(16.0) * atten;

        frag_color += diff + spec;
    }

    *out_frag_color = vec4(frag_color.x, frag_color.y, frag_color.z, 1.0);
}
