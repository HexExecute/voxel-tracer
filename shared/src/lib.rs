#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PackedNode(pub u32);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Material {
    pub albedo: [f32; 3],
    pub roughness: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Voxel {
    pub material: Material,
}
