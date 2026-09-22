use crate::Hash;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RootEntry {
    pub root: Hash,
    pub block_number: Option<u32>,
    pub block_hash: Option<Hash>,
    pub verified: bool,
}

pub struct RootTracker {
    entries: VecDeque<RootEntry>,
    cap: usize,
}

impl RootTracker {
    pub fn new(cap: usize) -> Self {
        Self { entries: VecDeque::with_capacity(cap), cap }
    }

    pub fn push(&mut self, entry: RootEntry) {
        if let Some(back) = self.entries.back_mut() {
            if back.root == entry.root {
                if entry.verified && !back.verified {
                    back.verified = entry.verified;
                    back.block_number = entry.block_number;
                    back.block_hash = entry.block_hash;
                }
                return;
            }
        }
        if self.entries.len() == self.cap {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn find(&self, root: &Hash) -> Option<&RootEntry> {
        if let Some(e) = self.entries.iter().rev().find(|e| &e.root == root && e.verified) {
            return Some(e);
        }
        self.entries.iter().rev().find(|e| &e.root == root)
    }

    pub fn latest_verified_block(&self) -> Option<u32> {
        self.entries.iter().rev()
            .find_map(|e| if e.verified { e.block_number } else { None })
    }

    pub fn len(&self) -> usize { self.entries.len() }
}
