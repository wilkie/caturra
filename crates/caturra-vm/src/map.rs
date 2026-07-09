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
//! The one divergence: with at least 8 keys colliding in a single bucket of
//! a table of 64 or more, a real `HashMap` converts that bucket to a
//! red-black tree and iterates it in tree order. caturra keeps the chain
//! order. Reaching it takes deliberately-crafted colliding keys.

use std::cell::OnceCell;
use std::collections::HashMap;

use crate::value::{HeapObject, HeapRef, JValue};

/// A key by value, as `Map` compares them: `equals` semantics, not
/// reference identity. Doubles and floats key on their raw bits, so
/// `-0.0` and `0.0` are distinct keys and `NaN` equals itself — exactly
/// what `Double.equals` does, and unlike `==` on the primitives.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
    Int(i32),
    Long(i64),
    /// `Double.doubleToLongBits`.
    Double(i64),
    /// `Float.floatToIntBits`.
    Float(i32),
    Str(Vec<u16>),
    /// A boxed `Boolean`. It is the one wrapper whose `hashCode` is not
    /// its value (1231/1237), so it needs its own key to land in the
    /// bucket the JDK would pick.
    Bool(bool),
    /// An object without value equality: identity, as Java's default
    /// `equals` does (caturra does not support overriding `equals`).
    Ref(HeapRef),
    Null,
}

impl MapKey {
    /// The key's `hashCode()`, matching the JDK's for every type whose
    /// keys caturra can compare by value.
    #[must_use]
    pub fn hash_code(&self) -> i32 {
        match self {
            MapKey::Int(v) => *v,
            // `Long.hashCode` / `Double.hashCode`: fold the high word down.
            MapKey::Long(v) => fold_to_int(*v),
            MapKey::Double(bits) => fold_to_int(*bits),
            MapKey::Float(bits) => *bits,
            MapKey::Bool(v) => {
                if *v {
                    1231
                } else {
                    1237
                }
            }
            MapKey::Str(units) => units
                .iter()
                .fold(0i32, |h, u| h.wrapping_mul(31).wrapping_add(i32::from(*u))),
            // Arbitrary in Java too, so no order is promised for object keys.
            MapKey::Ref(reference) => reference.cast_signed(),
            MapKey::Null => 0,
        }
    }

    /// `HashMap.hash(key)`: the hash code with its high bits spread down,
    /// so that tables smaller than 2^16 still see them. A null key hashes
    /// to 0 without spreading, and so always lands in bucket 0.
    fn spread(&self) -> u32 {
        if matches!(self, MapKey::Null) {
            return 0;
        }
        let h = self.hash_code().cast_unsigned();
        h ^ (h >> 16)
    }
}

/// One entry, remembering the key both as the value the program handed us
/// (what `keySet` and `toString` must show) and as its comparison key.
#[derive(Debug, Clone)]
struct Entry {
    key: JValue,
    comparable: MapKey,
    value: JValue,
}

/// A `java.util.HashMap` with the JDK's iteration order.
#[derive(Debug, Clone, Default)]
pub struct JavaHashMap {
    /// Entries in insertion order. Re-putting an existing key updates the
    /// value in place and keeps the original position, as Java does.
    entries: Vec<Entry>,
    /// Comparison key to index into `entries`, so `get`/`containsKey` are
    /// constant time rather than a scan.
    index: HashMap<MapKey, usize>,
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

    #[must_use]
    pub fn lookup(&self, key: &MapKey) -> Option<JValue> {
        self.index.get(key).map(|at| self.entries[*at].value)
    }

    #[must_use]
    pub fn contains_key(&self, key: &MapKey) -> bool {
        self.index.contains_key(key)
    }

    /// Insert or overwrite, returning the previous value. A re-put keeps the
    /// entry's position and its original key object, as Java's does.
    pub fn insert(&mut self, comparable: MapKey, key: JValue, value: JValue) -> Option<JValue> {
        if let Some(at) = self.index.get(&comparable) {
            let previous = self.entries[*at].value;
            self.entries[*at].value = value;
            return Some(previous);
        }
        if self.table_len == 0 {
            self.table_len = if self.threshold == 0 {
                DEFAULT_CAPACITY
            } else {
                self.threshold
            };
            self.threshold = self.table_len * 3 / 4;
        }
        self.index.insert(comparable.clone(), self.entries.len());
        self.entries.push(Entry {
            key,
            comparable,
            value,
        });
        if self.entries.len() > self.threshold {
            self.table_len *= 2;
            self.threshold = self.table_len * 3 / 4;
        }
        self.order.take();
        None
    }

    /// Remove a key, returning its value. The table never shrinks — Java's
    /// doesn't either, so the iteration order of what remains is unchanged.
    pub fn remove(&mut self, key: &MapKey) -> Option<JValue> {
        let at = self.index.remove(key)?;
        let removed = self.entries.remove(at);
        for slot in self.index.values_mut() {
            if *slot > at {
                *slot -= 1;
            }
        }
        self.order.take();
        Some(removed.value)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
        self.order.take();
    }

    #[must_use]
    pub fn contains_value(&self, value: JValue) -> bool {
        self.entries.iter().any(|entry| entry.value == value)
    }

    /// The entries in the JDK's iteration order: by bucket, and within a
    /// bucket by insertion.
    pub fn iteration_order(&self) -> &[usize] {
        self.order.get_or_init(|| {
            let mask = self.table_len.saturating_sub(1);
            let mut order: Vec<usize> = (0..self.entries.len()).collect();
            // A stable sort by bucket keeps each bucket's chain in
            // insertion order, which is where Java leaves it.
            order.sort_by_key(|at| self.entries[*at].comparable.spread() as usize & mask);
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

    /// Every entry in iteration order.
    #[must_use]
    pub fn entries_in_order(&self) -> Vec<(JValue, JValue)> {
        self.iteration_order()
            .iter()
            .map(|at| (self.entries[*at].key, self.entries[*at].value))
            .collect()
    }
}

/// `(int) (v ^ (v >>> 32))`, how `Long` and `Double` hash a 64-bit value.
fn fold_to_int(value: i64) -> i32 {
    (value ^ (value.cast_unsigned() >> 32).cast_signed()) as i32
}

/// Java's `HashMap.DEFAULT_INITIAL_CAPACITY`.
const DEFAULT_CAPACITY: usize = 16;

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

/// The comparison key for a value used as a map key. Strings compare by
/// content; everything else by its own `equals`.
#[must_use]
pub fn map_key(heap: &crate::value::Heap, value: JValue) -> MapKey {
    match value {
        JValue::Int(v) => MapKey::Int(v),
        JValue::Long(v) => MapKey::Long(v),
        // `floatToIntBits`/`doubleToLongBits` collapse every NaN payload to
        // one, which is why `Double.equals` finds NaN equal to itself where
        // `==` does not.
        JValue::Float(v) => MapKey::Float(if v.is_nan() {
            f32::NAN.to_bits().cast_signed()
        } else {
            v.to_bits().cast_signed()
        }),
        JValue::Double(v) => MapKey::Double(if v.is_nan() {
            f64::NAN.to_bits().cast_signed()
        } else {
            v.to_bits().cast_signed()
        }),
        JValue::Ref(None) => MapKey::Null,
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            Some(HeapObject::JavaString(units)) => MapKey::Str(units.clone()),
            Some(HeapObject::Boxed { class_name, value }) => {
                if class_name == "java/lang/Boolean" {
                    MapKey::Bool(*value != JValue::Int(0))
                } else {
                    map_key(heap, *value)
                }
            }
            _ => MapKey::Ref(reference),
        },
    }
}
