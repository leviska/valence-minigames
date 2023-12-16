use valence::{BlockPos, ChunkPos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Area {
    min: BlockPos,
    max: BlockPos,
}

pub fn block_pos_min(a: BlockPos, b: BlockPos) -> BlockPos {
    BlockPos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}

pub fn block_pos_max(a: BlockPos, b: BlockPos) -> BlockPos {
    BlockPos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}

impl Area {
    pub fn new(a: impl Into<BlockPos>, b: impl Into<BlockPos>) -> Self {
        let a = a.into();
        let b = b.into();
        Self {
            min: block_pos_min(a, b),
            max: block_pos_max(a, b),
        }
    }

    pub fn min(&self) -> BlockPos {
        self.min
    }

    pub fn max(&self) -> BlockPos {
        self.max
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: block_pos_min(self.min, other.min),
            max: block_pos_max(self.max, other.max),
        }
    }

    pub fn contains(&self, pos: impl Into<BlockPos>) -> bool {
        let pos = pos.into();

        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.y <= self.max.y
            && pos.z >= self.min.z
            && pos.z <= self.max.z
    }

    pub fn iter_block_pos(&self) -> impl Iterator<Item = BlockPos> {
        // copy so iterator won't capture self
        let min = self.min;
        let max = self.max;
        (min.x..=max.x)
            .flat_map(move |x| (min.y..=max.y).map(move |y| (x, y)))
            .flat_map(move |(x, y)| (min.z..=max.z).map(move |z| BlockPos::new(x, y, z)))
    }

    pub fn iter_block_pos_plane(&self) -> impl Iterator<Item = [i32; 2]> {
        // copy so iterator won't capture self
        let min = self.min;
        let max = self.max;
        (min.x..=max.x).flat_map(move |x| (min.z..=max.z).map(move |z| [x, z]))
    }

    pub fn iter_chunk_pos(&self) -> impl Iterator<Item = ChunkPos> {
        let min: ChunkPos = self.min.into();
        let max: ChunkPos = self.max.into();

        (min.x..=max.x).flat_map(move |x| (min.z..=max.z).map(move |z| ChunkPos::new(x, z)))
    }

    // Each axis in dir will be added to area
    // If axis < 0, it will add in negative direction
    pub fn expand(&self, dirs: impl Into<BlockPos>) -> Self {
        let dirs = dirs.into();
        let mut r = *self;
        if dirs.x < 0 {
            r.min.x += dirs.x;
        } else {
            r.max.x += dirs.x;
        }
        if dirs.y < 0 {
            r.min.y += dirs.y;
        } else {
            r.max.y += dirs.y;
        }
        if dirs.z < 0 {
            r.min.z += dirs.z;
        } else {
            r.max.z += dirs.z;
        }
        r
    }

    // Each axis in dir will be substracted to area
    // If axis < 0, it will sub in negative direction
    pub fn shrink(&self, dirs: impl Into<BlockPos>) -> Self {
        let dirs = dirs.into();
        let mut r = *self;
        if dirs.x < 0 {
            r.min.x -= dirs.x;
        } else {
            r.max.x -= dirs.x;
        }
        if dirs.y < 0 {
            r.min.y -= dirs.y;
        } else {
            r.max.y -= dirs.y;
        }
        if dirs.z < 0 {
            r.min.z -= dirs.z;
        } else {
            r.max.z -= dirs.z;
        }
        r
    }
}
