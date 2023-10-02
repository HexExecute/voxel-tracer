use rand::random;
use shared::{Material, PackedNode, Voxel};

pub enum Node {
    Branch { children: Box<[Self; 8]> },
    Leaf(Option<Voxel>),
}

pub struct PackedSparseVoxelOctree {
    pub nodes: Vec<[PackedNode; 8]>,
    pub voxels: Vec<Voxel>,
}

pub struct SparseVoxelOctree {
    root: Node,
    max_depth: u32,
}

impl SparseVoxelOctree {
    pub fn pack(&self) -> PackedSparseVoxelOctree {
        let mut nodes: Vec<[PackedNode; 8]> = vec![];
        let mut voxels: Vec<Voxel> = vec![];

        self.root.pack_traverse(&mut nodes, &mut voxels);

        PackedSparseVoxelOctree { voxels, nodes }
    }

    pub fn new(depth: u32) -> Self {
        Self {
            root: Node::new(depth),
            max_depth: depth,
        }
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&Voxel> {
        self.root.get(x, y, z, 2_u32.pow(self.max_depth - 1))
    }

    pub fn insert(&mut self, x: u32, y: u32, z: u32, node: Node, depth: u32) {
        if depth > self.max_depth {
            self.max_depth = depth;
        }
        self.root
            .insert(x, y, z, node, 2_u32.pow(self.max_depth - 1), depth);
    }
}

impl Node {
    pub fn new(depth: u32) -> Self {
        if depth == 0 {
            Node::Leaf(Some(Voxel {
                material: Material {
                    albedo: [random(), random(), random()],
                    roughness: 1.0,
                },
            }))
        } else {
            Node::Branch {
                children: Box::new(std::array::from_fn(|_| Node::new(depth - 1))),
            }
        }
    }

    fn pack_traverse(
        &self,
        nodes: &mut Vec<[PackedNode; 8]>,
        voxels: &mut Vec<Voxel>,
    ) -> PackedNode {
        match self {
            Node::Branch { children } => {
                let mut packed_children = [PackedNode(u32::MAX); 8];

                for (i, child) in children.iter().enumerate() {
                    packed_children[i] = child.pack_traverse(nodes, voxels);
                }

                let branch_idx = nodes.len();
                nodes.push(packed_children);

                PackedNode(branch_idx as u32)
            }
            Node::Leaf(Some(voxel)) => {
                let voxel_idx = voxels.len();
                voxels.push(*voxel);
                PackedNode(voxel_idx as u32 | 1 << 31)
            }
            Node::Leaf(None) => PackedNode(u32::MAX),
        }
    }

    pub fn get(&self, x: u32, y: u32, z: u32, size: u32) -> Option<&Voxel> {
        match self {
            Node::Leaf(voxel) => voxel.as_ref(),
            Node::Branch { children } => {
                let index = ((x >= size) as usize) << 0
                    | ((y >= size) as usize) << 1
                    | ((z >= size) as usize) << 2;

                children[index].get(x % size, y % size, z % size, size / 2)
            }
        }
    }

    pub fn insert(&mut self, x: u32, y: u32, z: u32, node: Node, size: u32, depth: u32) {
        if depth == 0 {
            *self = node;
            return;
        }

        match self {
            Node::Branch { children } => {
                let index = ((x >= size) as usize) << 0
                    | ((y >= size) as usize) << 1
                    | ((z >= size) as usize) << 2;

                children[index].insert(x % size, y % size, z % size, node, size / 2, depth - 1)
            }
            Node::Leaf(voxel) => {
                let voxel = voxel.clone();
                *self = Node::Branch {
                    children: Box::new(std::array::from_fn(|_| Node::Leaf(voxel))),
                };

                self.insert(x, y, z, node, size, depth);
            }
        }
    }
}
