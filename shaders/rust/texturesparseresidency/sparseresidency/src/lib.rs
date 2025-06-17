#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec4, Mat4, Vec2, Vec3, Vec4}, Image};
use spirv_std::image::SampledImage;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub view_pos: Vec4,
    pub lod_bias: f32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_lod_bias: &mut f32,
) {
    *out_uv = in_uv;
    *out_lod_bias = ubo.lod_bias;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_lod_bias: f32,
    #[spirv(descriptor_set = 0, binding = 1)] sampler_color: &SampledImage<Image!(2D, type=f32, sampled)>,
    out_frag_color: &mut Vec4,
) {
    // Get residency code for current texel
    let sparse_result = sampler_color.sample_bias_sparse(in_uv, in_lod_bias);
    
    let color = if sparse_result.is_resident() {
        sparse_result.value
    } else {
        vec4(0.0, 0.0, 0.0, 0.0)
    };
    
    *out_frag_color = color;
}