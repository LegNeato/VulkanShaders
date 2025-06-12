#![no_std]
#![cfg_attr(target_arch = "spirv", feature(lang_items, core_intrinsics))]

use spirv_std::glam::{vec4, Vec4};
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(
    out_color: &mut Vec4,
) {
    *out_color = vec4(1.0, 0.0, 0.0, 1.0);
}