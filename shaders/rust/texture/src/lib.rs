#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{Mat4, Vec2, Vec3, Vec4}, Image, Sampler, num_traits::Float};

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
    in_uv: Vec2,
    in_normal: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &Ubo,
    #[spirv(position)] out_position: &mut Vec4,
    out_uv: &mut Vec2,
    out_lod_bias: &mut f32,
    out_normal: &mut Vec3,
    out_view_vec: &mut Vec3,
    out_light_vec: &mut Vec3,
) {
    *out_uv = in_uv;
    *out_lod_bias = ubo.lod_bias;
    
    let world_pos = ubo.model * Vec4::new(in_pos.x, in_pos.y, in_pos.z, 1.0);
    
    *out_position = ubo.projection * world_pos;
    
    // Calculate normal matrix (inverse transpose of model matrix)
    // For simplicity, assuming uniform scaling (normal matrix = model matrix for rotation/translation)
    let normal_matrix = Mat4::from_cols(
        ubo.model.x_axis.truncate().extend(0.0),
        ubo.model.y_axis.truncate().extend(0.0),
        ubo.model.z_axis.truncate().extend(0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );
    
    *out_normal = (normal_matrix * Vec4::new(in_normal.x, in_normal.y, in_normal.z, 0.0)).truncate();
    
    let light_pos = Vec3::new(0.0, 0.0, 0.0);
    *out_light_vec = light_pos - world_pos.truncate();
    *out_view_vec = ubo.view_pos.truncate() - world_pos.truncate();
}

#[spirv(fragment)]
pub fn main_fs(
    in_uv: Vec2,
    in_lod_bias: f32,
    in_normal: Vec3,
    in_view_vec: Vec3,
    in_light_vec: Vec3,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] image: &Image!(2D, type=f32, sampled),
    out_frag_color: &mut Vec4,
) {
    let color = image.sample_bias(*sampler, in_uv, in_lod_bias);
    
    let n = in_normal.normalize();
    let l = in_light_vec.normalize();
    let v = in_view_vec.normalize();
    let r = reflect(-l, n);
    let diffuse = n.dot(l).max(0.0);
    let specular = r.dot(v).max(0.0).powf(16.0);
    
    let diffuse_color = Vec3::new(1.0, 1.0, 1.0) * diffuse;
    let specular_color = Vec3::splat(specular * color.w);
    
    *out_frag_color = Vec4::from((diffuse_color * color.truncate() + specular_color, 1.0));
}

fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * n.dot(i) * n
}