//! `java.util.concurrent.ConcurrentHashMap`'s table, node for node.
//!
//! A `HashMap`'s iteration order can be DERIVED from its entries (see
//! `map.rs`): a resize splits every chain while preserving relative order, so
//! the final table and the insertion order are enough. A `ConcurrentHashMap`'s
//! cannot. Its resize (`transfer`) walks each chain for the LAST RUN of nodes
//! that land in the same half, keeps that run as it is, and PREPENDS every node
//! before it — which reverses them. It also resizes one entry earlier than a
//! `HashMap` (when the count REACHES three quarters of the table, not when it
//! passes it). Measured on a JDK 11: of 4000 random maps, 2947 iterated in a
//! different order from a `HashMap` holding the same keys. So this keeps the
//! real chains and replays the JDK's operations on them.
//!
//! Nodes carry stable ids. A `ConcurrentHashMap` cursor is WEAKLY consistent:
//! it holds the node it will return next and follows that node's live `next`
//! link, never throwing `ConcurrentModificationException` — and a removed node
//! keeps its link, so a cursor positioned on one carries on from where it was.
//! What a removed node linked to is remembered for that.
//!
//! Not modelled: a bin of eight or more colliding keys, which the JDK turns
//! into a tree (or answers with a resize while the table is under 64).

use std::collections::HashMap;

/// `DEFAULT_CAPACITY`.
const DEFAULT_CAPACITY: usize = 16;
/// `MAXIMUM_CAPACITY`.
const MAXIMUM_CAPACITY: usize = 1 << 30;

/// `ConcurrentHashMap.spread`: the high half folded in, and the sign bit
/// cleared (a negative hash marks a special node there).
#[must_use]
pub fn spread(hash: i32) -> usize {
    let h = hash.cast_unsigned();
    ((h ^ (h >> 16)) & 0x7fff_ffff) as usize
}

/// `tableSizeFor`: the power of two at or above `c`.
fn table_size_for(c: usize) -> usize {
    c.max(1).next_power_of_two().min(MAXIMUM_CAPACITY)
}

/// The bins, by node id, and the resize threshold.
#[derive(Debug, Clone, Default)]
pub struct ChmTable {
    bins: Vec<Vec<u64>>,
    /// `sizeCtl`: before the table exists, the length to create it with (0
    /// for the default); after, the count at which it doubles.
    size_ctl: usize,
    /// For each node, the bin it is in now.
    bin_of: HashMap<u64, usize>,
    /// What a REMOVED node's `next` pointed at when it was unlinked — a
    /// cursor positioned on it follows this, as a JDK's does.
    ghost_next: HashMap<u64, Option<u64>>,
    /// Each live node's key hash, which is what a resize splits by.
    hash_of: HashMap<u64, i32>,
}

impl ChmTable {
    /// `new ConcurrentHashMap<>()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `new ConcurrentHashMap<>(map)`: `sizeCtl` starts at the default
    /// capacity, and the `putAll` that follows presizes for the source.
    #[must_use]
    pub fn for_copy() -> Self {
        Self {
            size_ctl: DEFAULT_CAPACITY,
            ..Self::default()
        }
    }

    /// `new ConcurrentHashMap<>(initialCapacity)`: JDK 11 sizes the table for
    /// the capacity at a load factor of two thirds —
    /// `tableSizeFor(c + c/2 + 1)`.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let size = if capacity >= MAXIMUM_CAPACITY >> 1 {
            MAXIMUM_CAPACITY
        } else {
            table_size_for(capacity + (capacity >> 1) + 1)
        };
        Self {
            size_ctl: size,
            ..Self::default()
        }
    }

    /// The table's length, zero before the first insertion.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bins.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bins.is_empty()
    }

    fn init(&mut self) {
        if self.bins.is_empty() {
            let n = if self.size_ctl > 0 {
                self.size_ctl
            } else {
                DEFAULT_CAPACITY
            };
            self.bins = vec![Vec::new(); n];
            self.size_ctl = n - (n >> 2);
        }
    }

    /// `putVal` (and the compute family): the node goes on the TAIL of its
    /// chain, and then `addCount` doubles the table while the count has
    /// reached `sizeCtl`.
    pub fn insert(&mut self, id: u64, hash: i32, count: usize) {
        self.init();
        let bin = spread(hash) & (self.bins.len() - 1);
        self.bins[bin].push(id);
        self.bin_of.insert(id, bin);
        self.hash_of.insert(id, hash);
        while count >= self.size_ctl && self.bins.len() < MAXIMUM_CAPACITY {
            self.transfer();
        }
    }

    /// `tryPresize(size)`, which `putAll` and the copy constructor run before
    /// they insert anything.
    pub fn presize(&mut self, size: usize) {
        let c = if size >= MAXIMUM_CAPACITY >> 1 {
            MAXIMUM_CAPACITY
        } else {
            table_size_for(size + (size >> 1) + 1)
        };
        loop {
            if self.bins.is_empty() {
                let n = self.size_ctl.max(c);
                self.bins = vec![Vec::new(); n];
                self.size_ctl = n - (n >> 2);
                continue;
            }
            if c <= self.size_ctl || self.bins.len() >= MAXIMUM_CAPACITY {
                break;
            }
            self.transfer();
        }
    }

    /// Unlink a node, remembering where it pointed.
    pub fn remove(&mut self, id: u64) {
        let Some(bin) = self.bin_of.remove(&id) else {
            return;
        };
        self.hash_of.remove(&id);
        let chain = &mut self.bins[bin];
        if let Some(at) = chain.iter().position(|node| *node == id) {
            let next = chain.get(at + 1).copied();
            chain.remove(at);
            self.ghost_next.insert(id, next);
        }
    }

    /// `clear()`: every node unlinked, the table kept at its size.
    pub fn clear(&mut self) {
        // A cleared bin's nodes keep their links, as unlinked ones do.
        for chain in &mut self.bins {
            for (at, node) in chain.iter().enumerate() {
                self.ghost_next.insert(*node, chain.get(at + 1).copied());
            }
            chain.clear();
        }
        self.bin_of.clear();
        self.hash_of.clear();
    }

    /// `transfer`: double the table. Each chain splits into the nodes that
    /// stay at `i` and those that move to `i + n`; the last RUN of nodes bound
    /// for one half is kept as it is, and every node before that run is
    /// pushed onto the FRONT of its half — in reverse.
    fn transfer(&mut self) {
        let n = self.bins.len();
        let hash_of = &self.hash_of;
        let mut next: Vec<Vec<u64>> = vec![Vec::new(); n << 1];
        for (i, chain) in self.bins.iter().enumerate() {
            if chain.is_empty() {
                continue;
            }
            let bit = |id: u64| spread(hash_of.get(&id).copied().unwrap_or(0)) & n;
            // The start of the last run of same-half nodes.
            let mut run_bit = bit(chain[0]);
            let mut last_run = 0;
            for (at, node) in chain.iter().enumerate().skip(1) {
                let b = bit(*node);
                if b != run_bit {
                    run_bit = b;
                    last_run = at;
                }
            }
            let mut low: Vec<u64> = Vec::new();
            let mut high: Vec<u64> = Vec::new();
            if run_bit == 0 {
                low.extend_from_slice(&chain[last_run..]);
            } else {
                high.extend_from_slice(&chain[last_run..]);
            }
            for node in &chain[..last_run] {
                if bit(*node) == 0 {
                    low.insert(0, *node);
                } else {
                    high.insert(0, *node);
                }
            }
            next[i] = low;
            next[i + n] = high;
        }
        self.bins = next;
        self.bin_of.clear();
        for (bin, chain) in self.bins.iter().enumerate() {
            for node in chain {
                self.bin_of.insert(*node, bin);
            }
        }
        let doubled = n << 1;
        self.size_ctl = doubled - (doubled >> 2);
    }

    /// Every node, in the order a traversal meets them.
    #[must_use]
    pub fn order(&self) -> Vec<u64> {
        self.bins.iter().flatten().copied().collect()
    }

    /// Where a cursor goes after `id`: the node's live successor in its
    /// chain — or, for a removed node, what it linked to when it went — and
    /// past a chain's end, the first node of the next bin that has one.
    /// `bin_hint` is the bin the cursor last saw the node in, for a node that
    /// is gone.
    #[must_use]
    pub fn successor(&self, id: u64, bin_hint: usize) -> Option<(u64, usize)> {
        let mut current = id;
        let mut bin = bin_hint;
        // Follow a removed node's remembered link, and any removed node after
        // it, until one that is still in the table — or the chain's end.
        loop {
            if let Some(&live_bin) = self.bin_of.get(&current) {
                bin = live_bin;
                let chain = &self.bins[bin];
                let at = chain.iter().position(|node| *node == current)?;
                if let Some(next) = chain.get(at + 1) {
                    return Some((*next, bin));
                }
                break;
            }
            match self.ghost_next.get(&current) {
                Some(Some(next)) => {
                    if self.bin_of.contains_key(next) {
                        return Some((*next, self.bin_of[next]));
                    }
                    current = *next;
                }
                _ => break,
            }
        }
        self.first_from(bin + 1)
    }

    /// The first node at or after bin `from`.
    #[must_use]
    pub fn first_from(&self, from: usize) -> Option<(u64, usize)> {
        self.bins
            .iter()
            .enumerate()
            .skip(from)
            .find_map(|(bin, chain)| chain.first().map(|node| (*node, bin)))
    }

    /// Whether a node is still in the table.
    #[must_use]
    pub fn contains(&self, id: u64) -> bool {
        self.bin_of.contains_key(&id)
    }

    /// The nodes that are remembered only because a cursor may be on one.
    pub fn ghosts(&self) -> impl Iterator<Item = u64> + '_ {
        self.ghost_next.keys().copied()
    }
}
