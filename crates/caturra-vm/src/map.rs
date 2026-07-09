//! `java.util.HashMap`, faithful down to its iteration order.
//!
//! A real `HashMap` iterates its bucket table front to back, and each
//! bucket's chain in insertion order. That order is not arbitrary — it is
//! a pure function of the keys' hash codes, the table's length, and the
//! order the keys were first inserted — and students see it every time
//! they print a map or walk `keySet()`. So we reproduce it exactly rather
//! than substituting a convenient insertion order.
//!
//! Entries are kept in insertion order and the bucket order is derived on
//! demand (and cached), which is equivalent: within one bucket of the final
//! table, Java's entries are always in insertion order, because appends go
//! to the chain's tail and a resize splits each chain while preserving
//! relative order.
//!
//! A key's hash code and its equality may both come from user code, which
//! only the interpreter can run. So this module stores each key's hash and
//! hands out the *candidates* sharing one; the caller decides which of them
//! the key actually equals. `HashMap.getNode` works the same way — compare
//! hashes first, then `equals`.
//!
//! Below a table length of 64, a bin of 8 makes Java resize rather than
//! treeify, which reshuffles every bucket; that is modelled. The one
//! divergence left: at 64 or more, such a bin really does become a red-black
//! tree, and iterates in tree order rather than chain order. Reaching it
//! takes deliberately-crafted colliding keys.

use std::cell::OnceCell;
use std::collections::HashMap;

use crate::value::JValue;

/// One mapping, remembering its key's `hashCode()` so that neither a lookup
/// nor the iteration order has to recompute it.
#[derive(Debug, Clone)]
struct Entry {
    key: JValue,
    hash: i32,
    value: JValue,
}

/// A `java.util.HashMap` with the JDK's iteration order.
#[derive(Debug, Clone, Default)]
pub struct JavaHashMap {
    /// Entries in insertion order. Re-putting an existing key updates the
    /// value in place and keeps the original position, as Java does.
    entries: Vec<Entry>,
    /// Key hash code to the entries carrying it. Two keys with the same hash
    /// need not be equal, so this narrows a lookup rather than answering it.
    index: HashMap<i32, Vec<usize>>,
    /// Java's `table.length`: zero until the first insertion allocates it.
    table_len: usize,
    /// Java's `threshold`: the size at which the table doubles. Before the
    /// table exists this doubles as the requested initial capacity.
    threshold: usize,
    /// The derived bucket order, computed on first use and dropped by any
    /// structural change. A cell, because reads (`toString`) hold the map
    /// immutably.
    order: OnceCell<Vec<usize>>,
}

impl JavaHashMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `new HashMap<>(initialCapacity)`. Java rounds the hint up to a power
    /// of two and parks it in `threshold` until the first insertion.
    #[must_use]
    pub fn with_capacity_hint(capacity: i32) -> Self {
        Self {
            threshold: table_size_for(capacity),
            ..Self::default()
        }
    }

    /// `new HashMap<>(otherMap)`. Java's `putMapEntries` pre-sizes the table
    /// to `tableSizeFor(size / 0.75 + 1)`, so the copy's iteration order can
    /// differ from the source's whenever that lands on a different length.
    #[must_use]
    pub fn sized_for(size: usize) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let fitting = (size as f64 / 0.75 + 1.0) as i32;
        Self {
            threshold: table_size_for(fitting),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries whose key hash is `hash`. The caller compares them with
    /// `equals` — which may be the user's — to find the real match.
    #[must_use]
    pub fn candidates(&self, hash: i32) -> Vec<usize> {
        self.index.get(&hash).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn key_at(&self, at: usize) -> JValue {
        self.entries[at].key
    }

    #[must_use]
    pub fn value_at(&self, at: usize) -> JValue {
        self.entries[at].value
    }

    /// Overwrite a mapping's value, returning the previous one. The entry
    /// keeps its position and its original key object, as Java's does.
    pub fn set_value_at(&mut self, at: usize, value: JValue) -> JValue {
        std::mem::replace(&mut self.entries[at].value, value)
    }

    /// Append a mapping whose key is known to be absent.
    pub fn insert_new(&mut self, hash: i32, key: JValue, value: JValue) {
        if self.table_len == 0 {
            self.table_len = if self.threshold == 0 {
                DEFAULT_CAPACITY
            } else {
                self.threshold
            };
            self.threshold = self.table_len * 3 / 4;
        }
        self.index.entry(hash).or_default().push(self.entries.len());
        self.entries.push(Entry { key, hash, value });

        // `treeifyBin`: a bin this long in a table this small makes Java grow
        // the table rather than grow a tree. That reshuffles every bucket, so
        // a map whose keys collide iterates differently than the load factor
        // alone would suggest — model it, or the order is wrong. `putVal`
        // acts when the chain it appended to *already held* `TREEIFY_THRESHOLD`
        // nodes, so the trigger is one past it.
        if self.table_len < MIN_TREEIFY_CAPACITY && self.bin_len(hash) > TREEIFY_THRESHOLD {
            self.grow();
        }
        if self.entries.len() > self.threshold {
            self.grow();
        }
        self.order.take();
    }

    /// Double the table, as `resize` does.
    fn grow(&mut self) {
        self.table_len *= 2;
        self.threshold = self.table_len * 3 / 4;
    }

    /// How many entries share the bucket that `hash` lands in. Only consulted
    /// while the table is smaller than `MIN_TREEIFY_CAPACITY`, where it is
    /// therefore short.
    fn bin_len(&self, hash: i32) -> usize {
        let mask = self.table_len - 1;
        let bucket = spread(hash) as usize & mask;
        self.entries
            .iter()
            .filter(|entry| spread(entry.hash) as usize & mask == bucket)
            .count()
    }

    /// Remove the mapping at `at`, returning its value. The table never
    /// shrinks — Java's doesn't either, so the iteration order of what
    /// remains is unchanged.
    pub fn remove_at(&mut self, at: usize) -> JValue {
        let removed = self.entries.remove(at);
        // Every later entry shifted down one, and this position is gone.
        for positions in self.index.values_mut() {
            positions.retain(|position| *position != at);
            for position in positions.iter_mut() {
                if *position > at {
                    *position -= 1;
                }
            }
        }
        self.index.retain(|_, positions| !positions.is_empty());
        self.order.take();
        removed.value
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
        self.order.take();
    }

    /// The entries in the JDK's iteration order: by bucket, and within a
    /// bucket by insertion.
    pub fn iteration_order(&self) -> &[usize] {
        self.order.get_or_init(|| {
            let mask = self.table_len.saturating_sub(1);
            let mut order: Vec<usize> = (0..self.entries.len()).collect();
            // A stable sort by bucket keeps each bucket's chain in
            // insertion order, which is where Java leaves it.
            order.sort_by_key(|at| spread(self.entries[*at].hash) as usize & mask);
            order
        })
    }

    /// The key/value pair at a position in iteration order.
    #[must_use]
    pub fn entry_at(&self, position: usize) -> Option<(JValue, JValue)> {
        let at = *self.iteration_order().get(position)?;
        let entry = &self.entries[at];
        Some((entry.key, entry.value))
    }

    /// Every entry in iteration order, with its key's hash. `new HashMap<>(map)`
    /// copies these straight across: the keys are already distinct, and their
    /// hash codes cannot have changed.
    #[must_use]
    pub fn hashed_entries_in_order(&self) -> Vec<(i32, JValue, JValue)> {
        self.iteration_order()
            .iter()
            .map(|at| {
                let entry = &self.entries[*at];
                (entry.hash, entry.key, entry.value)
            })
            .collect()
    }

    /// Every entry in iteration order.
    #[must_use]
    pub fn entries_in_order(&self) -> Vec<(JValue, JValue)> {
        self.iteration_order()
            .iter()
            .map(|at| (self.entries[*at].key, self.entries[*at].value))
            .collect()
    }
}

/// `HashMap.hash(key)`: the hash code with its high bits spread down, so
/// that tables smaller than 2^16 still see them. Java spreads a null key's
/// hash of 0 as well, which leaves it 0 — so null always lands in bucket 0.
fn spread(hash: i32) -> u32 {
    let hash = hash.cast_unsigned();
    hash ^ (hash >> 16)
}

/// Java's `HashMap.DEFAULT_INITIAL_CAPACITY`.
const DEFAULT_CAPACITY: usize = 16;

/// Java's `TREEIFY_THRESHOLD`: appending to a chain already this long makes
/// `HashMap` act.
const TREEIFY_THRESHOLD: usize = 8;

/// Java's `MIN_TREEIFY_CAPACITY`: below this the table doubles instead of
/// growing a red-black tree.
const MIN_TREEIFY_CAPACITY: usize = 64;

/// Java's `tableSizeFor`: the next power of two at or above `capacity`,
/// clamped to `MAXIMUM_CAPACITY`.
fn table_size_for(capacity: i32) -> usize {
    const MAXIMUM_CAPACITY: usize = 1 << 30;
    let Ok(capacity) = usize::try_from(capacity) else {
        return 1;
    };
    if capacity == 0 {
        return 1;
    }
    if capacity >= MAXIMUM_CAPACITY {
        return MAXIMUM_CAPACITY;
    }
    capacity.next_power_of_two()
}
