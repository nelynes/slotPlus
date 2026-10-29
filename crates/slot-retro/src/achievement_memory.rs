use std::ops::Range;

pub const ACHIEVEMENT_RAM_SIZE: usize = 0x58000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AchievementRegion {
    pub start: usize,
    pub len: usize,
}

/// Valid spans in a snapshot indexed by RetroAchievements addresses.
/// Fixed storage keeps per-frame snapshots free of metadata allocations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AchievementMemory {
    regions: [AchievementRegion; 16],
    count: usize,
}

impl AchievementMemory {
    pub fn new(ranges: impl IntoIterator<Item = Range<usize>>) -> Self {
        let mut memory = Self::default();
        for range in ranges {
            memory.push(range);
        }
        memory
    }

    pub(crate) fn push(&mut self, range: Range<usize>) {
        assert!(range.start <= range.end && range.end <= ACHIEVEMENT_RAM_SIZE);
        if range.is_empty() {
            return;
        }
        self.regions[self.count] = AchievementRegion {
            start: range.start,
            len: range.end - range.start,
        };
        self.count += 1;
    }

    pub fn regions(&self) -> &[AchievementRegion] {
        &self.regions[..self.count]
    }
}
