#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec4, Mat4, Vec2, Vec3, Vec4}};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
}


#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec4,
    _in_uv: Vec2,
    _in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_color: &mut Vec3,
) {
    *out_color = in_color;
    *out_position = ubo.projection * ubo.model * in_pos;
}

#[spirv(fragment)]
pub fn main_fs(
    in_color: Vec3,
    out_frag_color: &mut Vec4,
) {
    *out_frag_color = vec4(in_color.x, in_color.y, in_color.z, 1.0);
}
