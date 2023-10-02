#![cfg_attr(target_arch = "spirv", no_std)]

const SAMPLES: usize = 1;
const BOUNCES: usize = 16;

const CAMERA_ORIGIN: Vec3 = vec3(0.0, 0.0, 0.0);
const FOCAL_LENGTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 2.0;

const TREE_DEPTH: u32 = 3;

const EMPTY_MATERIAL: Material = Material {
    albedo: [0.0, 0.0, 0.0],
    roughness: 0.0,
};

#[allow(unused)]
use spirv_std::num_traits::Float;

use shared::{Material, PackedNode, ShaderConstants, Voxel};
use spirv_std::{
    glam::{ivec3, uvec3, vec2, vec3, vec4, BVec3, IVec3, UVec3, Vec3, Vec3Swizzles, Vec4},
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
    t: f32,
}

fn sphere(point: Vec3) -> f32 {
    (point - vec3(0.0, 0.0, -30.0)).length() - 15.0
}

fn less_than_equal(vec1: Vec3, vec2: Vec3) -> BVec3 {
    BVec3 {
        x: vec1.x <= vec2.x,
        y: vec1.y <= vec2.y,
        z: vec1.z <= vec2.z,
    }
}

struct GetResult {
    exists: bool,
    material: Material,
}

const EMPTY_GET_RESULT: GetResult = GetResult {
    exists: false,
    material: EMPTY_MATERIAL,
};

fn get(x: i32, y: i32, z: i32, nodes: &[[PackedNode; 8]], voxels: &[Voxel]) -> GetResult {
    if x < 0 || x >= 2_i32.pow(TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }
    if y < 0 || y >= 2_i32.pow(TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }
    if z < 0 || z >= 2_i32.pow(TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }

    let mut x = x as u32;
    let mut y = y as u32;
    let mut z = z as u32;

    let mut node = PackedNode(0);
    let mut s = 2_u32.pow(TREE_DEPTH - 1);

    loop {
        if node.is_empty() {
            return EMPTY_GET_RESULT;
        }
        if node.is_leaf() {
            return GetResult {
                exists: true,
                material: voxels[(node.0 - 1 << 31) as usize].material,
            };
        }

        // is branch
        let index = ((x >= s) as usize) << 0 | ((y >= s) as usize) << 1 | ((z >= s) as usize) << 2;

        node = nodes[node.0 as usize][index];

        x %= s;
        y %= s;
        z %= s;

        s /= 2;
    }

    // let mut node = PackedNode(0); // root node
    //
    // let mut s = 2_u32.pow(TREE_DEPTH);
    //
    // loop {
    //     if node.is_empty() {
    //         return EMPTY_GET_RESULT;
    //     }
    //
    //     if node.is_leaf() {
    //         return GetResult {
    //             exists: true,
    //             material: voxels[(node.0 - (1 << 31)) as usize].material,
    //         };
    //     }
    //
    //     let index = ((x >= s as i32) as usize) << 0
    //         | ((y >= s as i32) as usize) << 1
    //         | ((z >= s as i32) as usize) << 2;
    //
    //     node = nodes[node.0 as usize][index];
    //
    //     x %= s as i32;
    //     y %= s as i32;
    //     z %= s as i32;
    //
    //     s /= 2;
    // }
}

impl Ray {
    fn traverse(&mut self, nodes: &[[PackedNode; 8]], voxels: &[Voxel]) -> HitResult {
        let delta_dist = 1.0 / self.direction.abs();
        let ray_step = self.direction.signum().as_ivec3();
        let mut map_pos = self.origin.floor().as_ivec3();
        let mut side_dist = (self.direction.signum() * (map_pos.as_vec3() - self.origin)
            + (self.direction * 0.5)
            + 0.5)
            * delta_dist;

        let mut mask = BVec3 {
            x: false,
            y: false,
            z: false,
        };

        let mut fmask = vec3(0.0, 0.0, 0.0);

        for _ in 0..30 {
            // self.t += sphere(self.origin);
            self.origin += self.direction;

            // if sphere(map_pos.as_vec3()) < 0.01 {
            // let uposition = uvec3(map_pos.x as u32, map_pos.y as u32, map_pos.z as u32);
            // let uposition = map_pos.as_uvec3();
            let get_result = get(map_pos.x, map_pos.y, map_pos.z + 20, nodes, voxels);
            if get_result.exists {
                let d = (fmask * (side_dist - delta_dist)).length() / 100.0;

                return HitResult {
                    exists: true,
                    position: Vec3::ZERO,
                    normal: Vec3::ZERO,
                    material: get_result.material,
                };
            }

            mask = less_than_equal(side_dist.xyz(), side_dist.yzx().min(side_dist.zxy()));
            fmask = vec3(
                mask.x as i32 as f32,
                mask.y as i32 as f32,
                mask.z as i32 as f32,
            );
            side_dist += fmask * delta_dist;
            map_pos += fmask.as_ivec3() * ray_step;
        }

        // let d = (fmask * (side_dist - delta_dist)).length() / 100.0;

        HitResult {
            exists: false,
            position: Vec3::ZERO,
            normal: Vec3::ZERO,
            material: Material {
                albedo: [0.0, 0.0, 0.0],
                roughness: 0.0,
            },
        }
    }

    fn color(&mut self, nodes: &[[PackedNode; 8]], voxels: &[Voxel]) -> Vec3 {
        let hit_result = self.traverse(nodes, voxels);

        if hit_result.exists {
            vec3(
                hit_result.material.albedo[0],
                hit_result.material.albedo[1],
                hit_result.material.albedo[2],
            )
        } else {
            Vec3::ZERO
        }
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

    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] nodes: &[[PackedNode; 8]],
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

        let mut ray = Ray {
            origin: CAMERA_ORIGIN,
            direction: (pixel_center - CAMERA_ORIGIN).normalize(),
            t: 0.0,
        };

        color += ray.color(nodes, voxels);
    }

    color /= SAMPLES as f32;

    *output = vec4(color.x, color.y, color.z, 1.0);
}
