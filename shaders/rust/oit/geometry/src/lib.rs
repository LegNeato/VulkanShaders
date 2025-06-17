#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec4, Mat4, Vec3, Vec4, IVec2}, arch::atomic_i_add, Image};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub color: Vec4,
    pub depth: f32,
    pub next: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RenderPassUbo {
    pub projection: Mat4,
    pub view: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConsts {
    pub model: Mat4,
    pub color: Vec4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GeometrySbo {
    pub count: u32,
    pub max_node_count: u32,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] render_pass_ubo: &RenderPassUbo,
    #[spirv(push_constant)] push_consts: &PushConsts,
    #[spirv(position)] out_position: &mut Vec4,
) {
    let pvm = render_pass_ubo.projection * render_pass_ubo.view * push_consts.model;
    *out_position = pvm * vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] geometry_sbo: &mut GeometrySbo,
    #[spirv(descriptor_set = 0, binding = 2)] head_index_image: &Image!(2D, type=u32, sampled=false),
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)] nodes: &mut [Node],
    #[spirv(push_constant)] push_consts: &PushConsts,
) {
    // Increase the node count
    let node_idx = unsafe { atomic_i_add(&mut geometry_sbo.count, 1) };

    // Check LinkedListSBO is full
    if node_idx < geometry_sbo.max_node_count {
        // Exchange new head index and previous head index
        let coord = IVec2::new(frag_coord.x as i32, frag_coord.y as i32);
        let prev_head_idx = unsafe { 
            use spirv_std::memory::{Scope, Semantics};
            use spirv_std::arch::atomic_exchange;
            // For images, we need to use a different approach
            // Since atomic_ptr is not available, we'll need to use a workaround
            // The GLSL uses imageAtomicExchange which is not directly available in rust-gpu
            // We'll use a simple read for now and note this limitation
            let current = head_index_image.read(coord).x;
            head_index_image.write(coord, node_idx.into());
            current
        };

        // Store node data
        nodes[node_idx as usize].color = push_consts.color;
        nodes[node_idx as usize].depth = frag_coord.z;
        nodes[node_idx as usize].next = prev_head_idx;
    }
}