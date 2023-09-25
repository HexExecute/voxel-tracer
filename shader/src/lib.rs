#![cfg_attr(target_arch = "spirv", no_std)]

const SAMPLES: usize = 16;
const BOUNCES: usize = 16;

const CAMERA_ORIGIN: Vec3 = vec3(0.0, 0.0, 0.0);
const FOCAL_LENGTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 2.0;

#[allow(unused)]
use spirv_std::num_traits::Float;

use shared::{Material, ShaderConstants, Voxel};
use spirv_std::{
    glam::{vec2, vec3, vec4, Vec3, Vec4},
    spirv,
};

struct HitResult {
    exists: bool,
    position: Vec3,
    normal: Vec3,
    material: Material,
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

fn traverse() -> HitResult {
    todo!()
}

impl Ray {
    fn color(&self) -> Vec3 {
        // let hit_result =
        todo!()
    }
}

// Vertex
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let uv_out = uv * 2.0 - 1.0;

    *out_pos = vec4(uv_out.x, uv_out.y, 0.0, 1.0);
}

// Fragment
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] nodes: &[[u32; 8]],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] voxels: &[Voxel],

    output: &mut Vec4,
) {
    let aspect_ratio = constants.width as f32 / constants.height as f32;
    let viewport_width = VIEWPORT_HEIGHT * aspect_ratio;

    let viewport_u = vec3(viewport_width, 0.0, 0.0);
    let viewport_v = vec3(0.0, -VIEWPORT_HEIGHT, 0.0);

    let pixel_delta_u = viewport_u / constants.width as f32;
    let pixel_delta_v = viewport_v / constants.height as f32;

    let viewport_upper_left =
        CAMERA_ORIGIN - vec3(0.0, 0.0, FOCAL_LENGTH) - viewport_u / 2.0 - viewport_v / 2.0;

    let mut color = vec3(0.0, 0.0, 0.0);

    for _ in 0..SAMPLES {
        let pixel_center =
            viewport_upper_left + frag_coord.x * pixel_delta_u + frag_coord.y * pixel_delta_v;

        let ray = Ray {
            origin: CAMERA_ORIGIN,
            direction: (pixel_center - CAMERA_ORIGIN).normalize(),
        };

        // color += ray.color();
    }

    color /= SAMPLES as f32;

    color.x = voxels[0].material.roughness;

    *output = vec4(color.x, color.y, color.z, 1.0);
}
