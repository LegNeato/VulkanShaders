#![cfg_attr(target_arch = "spirv", no_std)]
#![feature(asm_experimental_arch)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{
    glam::{vec3, vec4, Mat3, Mat4, Vec3, Vec4},
    macros::debug_printf,
    num_traits::float::Float,
    spirv,
};

#[repr(C)]
pub struct UBO {
    projection: Mat4,
    model: Mat4,
    light_pos: Vec4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] _vertex_index: i32,
    in_pos: Vec3,
    in_normal: Vec3,
    in_color: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    #[spirv(position)] out_position: &mut Vec4,
    out_normal: &mut Vec3,
    out_color: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_normal = in_normal;
    *out_color = in_color;
    *out_position = ubo.projection * ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    let pos = ubo.model * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);

    // Output the vertex position using debug printf
    unsafe {
        debug_printf!("Position = %v4f", pos);
    }

    let model_mat3 = Mat3::from_cols(
        ubo.model.x_axis.truncate(),
        ubo.model.y_axis.truncate(),
        ubo.model.z_axis.truncate(),
    );

    *out_normal = model_mat3 * in_normal;
    let l_pos = model_mat3 * ubo.light_pos.truncate();
    *out_light_vec = l_pos - pos.truncate();
    *out_view_vec = -pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_normal: Vec3,
    in_color: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] _sampler_color_map: &spirv_std::Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    // Desaturate color
    let gray = in_color.x * 0.2126 + in_color.y * 0.7152 + in_color.z * 0.0722;
    let desaturated = vec3(gray, gray, gray);
    let color = in_color.lerp(desaturated, 0.65);

    // High ambient colors because mesh materials are pretty dark
    let ambient = color;
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = (-l).reflect(n);

    let diffuse = n.dot(l).max(0.0) * color;
    let specular = r.dot(v).max(0.0).powf(16.0) * vec3(0.75, 0.75, 0.75);

    let base_color = ambient + diffuse * 1.75 + specular;
    *out_frag_color = vec4(base_color.x, base_color.y, base_color.z, 1.0);

    // Toon shading
    let intensity = n.dot(l);
    let mut shade = 1.0;
    if intensity < 0.5 {
        shade = 0.75;
    }
    if intensity < 0.35 {
        shade = 0.6;
    }
    if intensity < 0.25 {
        shade = 0.5;
    }
    if intensity < 0.1 {
        shade = 0.25;
    }

    let final_color = in_color * 3.0 * shade;
    *out_frag_color = vec4(final_color.x, final_color.y, final_color.z, 1.0);
}
