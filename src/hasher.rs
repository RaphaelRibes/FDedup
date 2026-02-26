use rustc_hash::FxHashSet;
use std::hash::Hash;
use xxhash_rust::xxh3::{xxh3_128, xxh3_64};

pub struct HashChecker<T>
where
    T: Hash + Eq
{
    memoire: FxHashSet<T>,
}

impl<T> HashChecker<T>
where
    T: Hash + Eq
{
    /// Initialise la structure avec une capacité générique.
    pub fn new(capacite_estimee: usize) -> Self {
        Self {
            memoire: FxHashSet::with_capacity_and_hasher(
                capacite_estimee,
                Default::default()
            ),
        }
    }

    /// Vérifie si la clef est en mémoire.
    pub fn check(&mut self, hash: T) -> bool {
        self.memoire.insert(hash)
    }
}

pub(crate) trait SequenceHasher: Hash + Eq + Copy + Send + Sync {
    fn hash_seq(seq: &[u8]) -> Self;
}

impl SequenceHasher for u64 {
    #[inline(always)]
    fn hash_seq(seq: &[u8]) -> Self { xxh3_64(seq) }
}

impl SequenceHasher for u128 {
    #[inline(always)]
    fn hash_seq(seq: &[u8]) -> Self { xxh3_128(seq) }
}

pub(crate) enum HashType {
    XXH3_64,
    XXH3_128,
}