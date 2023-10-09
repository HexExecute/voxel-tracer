#![cfg_attr(target_arch = "spirv", no_std)]

const SAMPLES: usize = 1;
const BOUNCES: usize = 16;
const TRAVERSAL_STEPS: usize = 40;

const CAMERA_ORIGIN: Vec3 = vec3(0.0, 0.0, 0.0);
const FOCAL_LENGTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 2.0;

const EMPTY_MATERIAL: Material = Material {
    albedo: [0.0, 0.0, 0.0],
    roughness: 0.0,
};

#[allow(unused)]
use spirv_std::num_traits::Float;

use shared::{Material, PackedNode, ShaderConstants, Voxel, TREE_DEPTH};
use spirv_std::{
    glam::{vec2, vec3, vec4, BVec3, Vec3, Vec3Swizzles, Vec4},
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

fn less_than_equal(vec1: Vec3, vec2: Vec3) -> BVec3 {
    BVec3 {
        x: vec1.x <= vec2.x,
        y: vec1.y <= vec2.y,
        z: vec1.z <= vec2.z,
    }
}

struct GetResult {
    is_branch: bool,
    is_higher: bool,
    exists: bool,
    material: Material,
}

const EMPTY_GET_RESULT: GetResult = GetResult {
    is_branch: false,
    is_higher: false,
    exists: false,
    material: EMPTY_MATERIAL,
};

fn get(
    x: i32,
    y: i32,
    z: i32,
    depth: usize,
    nodes: &[[PackedNode; 8]],
    voxels: &[Voxel],
    root: PackedNode,
) -> GetResult {
    if x < 0 || x >= (1 << TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }
    if y < 0 || y >= (1 << TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }
    if z < 0 || z >= (1 << TREE_DEPTH) {
        return EMPTY_GET_RESULT;
    }

    let mut x = x as u32;
    let mut y = y as u32;
    let mut z = z as u32;

    let mut node = root;
    let mut s = 2_u32.pow(TREE_DEPTH - 1);

    for i in 0..(depth + 1) {
        if node.is_empty() {
            return EMPTY_GET_RESULT;
        }
        if node.is_leaf() {
            return GetResult {
                is_branch: false,
                is_higher: i < depth,
                exists: true,
                material: voxels[(node.0 & !(1 << 31)) as usize].material,
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

    GetResult {
        is_branch: true,
        is_higher: false,
        exists: true,
        material: EMPTY_MATERIAL,
    }
}

impl Ray {
    fn traverse(
        &mut self,
        nodes: &[[PackedNode; 8]],
        voxels: &[Voxel],
        root: PackedNode,
    ) -> HitResult {
        // origin_location + origin_offset = origin
        let mut origin_location = self.origin.trunc().as_ivec3();
        // let origin_offset = self.origin - origin_location.as_vec3();

        let delta_distance = (self.direction.length() / self.direction).abs();
        let mut side_distance = (self.direction.signum()
            * (origin_location.as_vec3() - self.origin)
            + (self.direction.signum() * 0.5)
            + 0.5)
            * delta_distance;

        // imgaginary stack of parent nodes though no real data is necessary
        // we start at maximum depth and then work our way up if we need to
        let mut stack_length = 4;

        for _ in 0..TRAVERSAL_STEPS {
            let get_result = get(
                origin_location.x - 3,
                origin_location.y + 5,
                origin_location.z + 25,
                stack_length,
                nodes,
                voxels,
                root,
            );

            if get_result.is_branch {
                stack_length += 1;
                continue;
            }

            if get_result.is_higher {
                stack_length -= 1;
                continue;
            }

            if get_result.exists {
                return HitResult {
                    exists: true,
                    material: get_result.material,
                    position: Vec3::ZERO, // not setup yet
                    normal: Vec3::ZERO,   // not setup yet
                };
            }

            // move ray
            let ray_step_size: i32 = 1 << (TREE_DEPTH - stack_length as u32);
            let ray_step = self.direction.signum().as_ivec3() * ray_step_size;

            let mask = less_than_equal(
                side_distance.xyz(),
                side_distance.yzx().min(side_distance.zxy()),
            );

            let fmask = vec3(
                if mask.x { 1.0 } else { 0.0 },
                if mask.y { 1.0 } else { 0.0 },
                if mask.z { 1.0 } else { 0.0 },
            );

            side_distance += fmask * delta_distance;
            origin_location += fmask.as_ivec3() * ray_step;
        }

        // didn't hit anything
        HitResult {
            exists: false,
            material: Material {
                albedo: [0.0, 0.0, 0.0],
                roughness: 1.0,
            },
            position: Vec3::ZERO,
            normal: Vec3::ZERO,
        }
    }

    fn color(&mut self, nodes: &[[PackedNode; 8]], voxels: &[Voxel], root: PackedNode) -> Vec3 {
        let hit_result = self.traverse(nodes, voxels, root);

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
        };

        color += ray.color(nodes, voxels, constants.root_node);
    }

    color /= SAMPLES as f32;

    *output = vec4(color.x, color.y, color.z, 1.0);
}
