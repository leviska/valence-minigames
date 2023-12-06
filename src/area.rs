use valence::{BlockPos, ChunkPos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Area {
    pub a: BlockPos,
    pub b: BlockPos,
}

impl Area {
    pub fn new(a: impl Into<BlockPos>, b: impl Into<BlockPos>) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }

    pub fn contains(&self, pos: impl Into<BlockPos>) -> bool {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );
        let pos = pos.into();

        pos.x >= min.x
            && pos.x <= max.x
            && pos.y >= min.y
            && pos.y <= max.y
            && pos.z >= min.z
            && pos.z <= max.z
    }

    pub fn iter_block_pos(&self) -> impl Iterator<Item = BlockPos> {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );

        (min.x..=max.x)
            .flat_map(move |x| (min.y..=max.y).map(move |y| (x, y)))
            .flat_map(move |(x, y)| (min.z..=max.z).map(move |z| BlockPos::new(x, y, z)))
    }

    pub fn iter_block_pos_plane(&self) -> impl Iterator<Item = [i32; 2]> {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );

        (min.x..=max.x).flat_map(move |x| (min.z..=max.z).map(move |z| [x, z]))
    }

    pub fn iter_chunk_pos(&self) -> impl Iterator<Item = ChunkPos> {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );
        let min: ChunkPos = min.into();
        let max: ChunkPos = max.into();

        (min.x..=max.x).flat_map(move |x| (min.z..=max.z).map(move |z| ChunkPos::new(x, z)))
    }
}
