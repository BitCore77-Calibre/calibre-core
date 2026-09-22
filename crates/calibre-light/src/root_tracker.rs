use crate::Hash;
use std::collections::VecDeque;

pub struct RootTracker {
    buf: VecDeque<Hash>,
    cap: usize,
}

impl RootTracker {
    pub fn new(cap: usize) -> Self {
        Self { buf: VecDeque::with_capacity(cap), cap }
    }

    pub fn push(&mut self, root: Hash) {
        if self.buf.back() == Some(&root) { return; }
        if self.buf.len() == self.cap { self.buf.pop_front(); }
        self.buf.push_back(root);
    }

    pub fn contains(&self, root: &Hash) -> bool {
        self.buf.iter().any(|r| r == root)
    }

    pub fn len(&self) -> usize { self.buf.len() }
}
