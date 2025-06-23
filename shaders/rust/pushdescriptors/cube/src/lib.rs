#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec4, Mat4, Vec2, Vec3, Vec4}, Image};
use spirv_std::image::SampledImage;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UboScene {
    pub projection: Mat4,
    pub view: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UboModel {
    pub local: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo_camera: &UboScene,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo_model: &UboModel,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_uv: &mut Vec2,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_uv = in_uv;
    *out_position = ubo_camera.projection * ubo_camera.view * ubo_model.local * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    _in_normal: Vec3,
    in_color: Vec3,
    in_uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 2)] sampler_color_map: &SampledImage<Image!(2D, type=f32, sampled)>,
    out_frag_color: &mut Vec4,
) {
    let texture_color = sampler_color_map.sample(in_uv);
    *out_frag_color = texture_color * vec4(in_color.x, in_color.y, in_color.z, 1.0);
}