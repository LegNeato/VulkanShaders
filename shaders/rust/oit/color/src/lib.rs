#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::missing_safety_doc)]

use spirv_std::{spirv, glam::{vec2, vec4, Vec4, IVec2}, Image};

const MAX_FRAGMENT_COUNT: usize = 128;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub color: Vec4,
    pub depth: f32,
    pub next: u32,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: u32,
    #[spirv(position)] out_position: &mut Vec4,
) {
    let uv = vec2(
        ((vertex_index << 1) & 2) as f32,
        (vertex_index & 2) as f32,
    );
    *out_position = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(descriptor_set = 0, binding = 0)] head_index_image: &Image!(2D, type=u32, sampled=false),
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] nodes: &[Node],
    out_frag_color: &mut Vec4,
) {
    let mut fragments = [Node { 
        color: Vec4::ZERO, 
        depth: 0.0, 
        next: 0 
    }; MAX_FRAGMENT_COUNT];
    let mut count = 0;

    let coord = IVec2::new(frag_coord.x as i32, frag_coord.y as i32);
    let mut node_idx = head_index_image.read(coord).x;

    while node_idx != 0xffffffff && count < MAX_FRAGMENT_COUNT {
        fragments[count] = nodes[node_idx as usize];
        node_idx = fragments[count].next;
        count += 1;
    }
    
    // Do the insertion sort
    for i in 1..count {
        let insert = fragments[i];
        let mut j = i;
        while j > 0 && insert.depth > fragments[j - 1].depth {
            fragments[j] = fragments[j - 1];
            j -= 1;
        }
        fragments[j] = insert;
    }

    // Do blending
    let mut color = vec4(0.025, 0.025, 0.025, 1.0);
    for i in 0..count {
        let frag_color = fragments[i].color;
        color = color.lerp(frag_color, frag_color.w);
    }

    *out_frag_color = color;
}