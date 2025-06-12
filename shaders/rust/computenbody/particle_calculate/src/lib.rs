#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{vec3, vec4, Vec3, Vec4},
    spirv,
    num_traits::Float,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Particle {
    pub pos: [f32; 4],  // xyz = position, w = mass
    pub vel: [f32; 4],  // xyz = velocity, w = gradient texture position
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UBO {
    pub delta_t: f32,
    pub particle_count: u32,
    pub gravity: f32,
    pub power: f32,
    pub soften: f32,
}

#[spirv(compute(threads(256)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] global_id: spirv_std::glam::UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] particles: &mut [Particle],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] ubo: &UBO,
) {
    let index = global_id.x as usize;
    
    if index >= ubo.particle_count as usize {
        return;
    }
    
    let position = vec4(particles[index].pos[0], particles[index].pos[1], particles[index].pos[2], particles[index].pos[3]);
    let mut velocity = vec4(particles[index].vel[0], particles[index].vel[1], particles[index].vel[2], particles[index].vel[3]);
    let mut acceleration = vec4(0.0, 0.0, 0.0, 0.0);
    
    // Calculate forces from all other particles (simplified O(N²) approach)
    for i in 0..ubo.particle_count as usize {
        if i == index {
            continue; // Skip self-interaction
        }
        
        let other = vec4(particles[i].pos[0], particles[i].pos[1], particles[i].pos[2], particles[i].pos[3]);
        let len = vec3(other.x - position.x, other.y - position.y, other.z - position.z);
        let distance_sq = len.dot(len) + ubo.soften;
        let distance = distance_sq.sqrt();
        let force_magnitude = ubo.gravity * other.w / distance_sq.powf(ubo.power / 2.0);
        
        acceleration = acceleration + vec4(
            len.x * force_magnitude, 
            len.y * force_magnitude, 
            len.z * force_magnitude, 
            0.0
        );
    }
    
    // Update velocity with acceleration
    velocity = vec4(
        velocity.x + acceleration.x * ubo.delta_t,
        velocity.y + acceleration.y * ubo.delta_t,
        velocity.z + acceleration.z * ubo.delta_t,
        velocity.w
    );
    
    // Update gradient texture position for visual effects
    velocity.w += 0.1 * ubo.delta_t;
    if velocity.w > 1.0 {
        velocity.w -= 1.0;
    }
    
    particles[index].vel = [velocity.x, velocity.y, velocity.z, velocity.w];
}