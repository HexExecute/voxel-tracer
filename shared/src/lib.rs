#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Eq, Debug)]
pub struct PackedNode(pub u32);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub root_node: PackedNode,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Material {
    pub albedo: [f32; 3],
    pub roughness: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Voxel {
    pub material: Material,
}

impl PackedNode {
    pub fn is_leaf(&self) -> bool {
        self.0 >= (1 << 31)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == u32::MAX
    }
}

pub const TREE_DEPTH: u32 = 4;
