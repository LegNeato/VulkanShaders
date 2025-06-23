#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec3, vec4, Mat4, Vec2, Vec3, Vec4}, Image};
use spirv_std::image::SampledImage;
use spirv_std::arch::Derivative;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub outline_color: Vec4,
    pub outline_width: f32,
    pub outline: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
) {
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<Image!(2D, type=f32, sampled)>,
    out_frag_color: &mut Vec4,
) {
    let distance = sampler_color.sample(in_uv).w;
    let smooth_width = distance.fwidth();
    let mut alpha = smooth_step(0.5 - smooth_width, 0.5 + smooth_width, distance);
    let mut rgb = vec3(alpha, alpha, alpha);
    
    if ubo.outline > 0.0 {
        let w = 1.0 - ubo.outline_width;
        alpha = smooth_step(w - smooth_width, w + smooth_width, distance);
        rgb += rgb.lerp(ubo.outline_color.truncate(), alpha);
    }
    
    *out_frag_color = vec4(rgb.x, rgb.y, rgb.z, alpha);
}