mod pareto_front_builder;
pub use pareto_front_builder::{ParetoFrontBuilder, ParetoValue};
use simulator::Settings;

pub struct NamedTimer {
    name: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    timer: std::time::Instant,
}

impl NamedTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            #[cfg(not(target_arch = "wasm32"))]
            timer: std::time::Instant::now(),
        }
    }
}

impl Drop for NamedTimer {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        eprintln!("{}: (timer not available on WASM)", self.name);
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "{}: {} seconds",
            self.name,
            self.timer.elapsed().as_secs_f32()
        );
    }
}

pub struct Backtracking<T: Copy> {
    entries: Vec<(T, u32)>,
}

impl<T: Copy> Backtracking<T> {
    pub const SENTINEL: u32 = u32::MAX;

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, mut index: u32) -> impl Iterator<Item = T> {
        let mut items = Vec::new();
        while index != Self::SENTINEL {
            items.push(self.entries[index as usize].0);
            index = self.entries[index as usize].1;
        }
        items.into_iter().rev()
    }

    pub fn push(&mut self, item: T, parent: u32) -> u32 {
        self.entries.push((item, parent));
        self.entries.len() as u32 - 1
    }
}

impl<T: Copy> Drop for Backtracking<T> {
    fn drop(&mut self) {
        dbg!(self.entries.len());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    pub quality: u16,
    pub duration: u8,
    pub steps: u8,
    pub quality_overflow: u16,
}

impl Score {
    pub fn new(quality: u16, duration: u8, steps: u8, settings: &Settings) -> Self {
        Self {
            quality: std::cmp::min(settings.max_quality, quality),
            duration,
            steps,
            quality_overflow: quality.saturating_sub(settings.max_quality),
        }
    }
}

impl std::cmp::PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl std::cmp::Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality
            .cmp(&other.quality)
            .then(other.duration.cmp(&self.duration))
            .then(other.steps.cmp(&self.steps))
            .then(self.quality_overflow.cmp(&other.quality_overflow))
    }
}

impl radix_heap::Radix for Score {
    const RADIX_BITS: u32 = 48;
    fn radix_similarity(&self, other: &Self) -> u32 {
        if self.quality != other.quality {
            self.quality.radix_similarity(&other.quality)
        } else if self.duration != other.duration {
            self.duration.radix_similarity(&other.duration) + 16
        } else if self.steps != other.steps {
            self.steps.radix_similarity(&other.steps) + 24
        } else {
            self.quality_overflow
                .radix_similarity(&other.quality_overflow)
                + 32
        }
    }
}
