#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{mat3, vec3, vec4, Mat4, Vec2, Vec3, Vec4}, num_traits::Float};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Ubo {
    pub projection: Mat4,
    pub model: Mat4,
    pub light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_normal: Vec3,
    in_uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_uv = in_uv;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    
    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    let normalized_normal = in_normal.normalize();
    *out_normal = (ubo.model.inverse().transpose() * vec4(normalized_normal.x, normalized_normal.y, normalized_normal.z, 0.0)).truncate();
    let model_mat3 = mat3(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );
    let l_pos = model_mat3 * ubo.light_pos.truncate();
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = -pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_uv: Vec2,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    out_frag_color: &mut Vec4,
) {
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);
    let diffuse = n.dot(l).max(0.0);
    let specular = r.dot(v).max(0.0).powf(1.0);
    let color = vec3(0.25, 0.25, 0.25) * (diffuse + specular);
    *out_frag_color = vec4(color.x, color.y, color.z, 1.0);
}