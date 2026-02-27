use rustc_hash::FxHashSet;
use std::hash::Hash;
use xxhash_rust::xxh3::{xxh3_64, xxh3_128};

pub struct VerificateurHachage<T>
where
    T: Hash + Eq,
{
    memoire: FxHashSet<T>,
}

impl<T> VerificateurHachage<T>
where
    T: Hash + Eq,
{
    /// Initialise la structure avec une capacité générique.
    pub fn nouveau(capacite_estimee: usize) -> Self {
        Self {
            memoire: FxHashSet::with_capacity_and_hasher(capacite_estimee, Default::default()),
        }
    }

    /// Vérifie si la clef est en mémoire.
    pub fn verifier(&mut self, hachage: T) -> bool {
        self.memoire.insert(hachage)
    }
}

pub(crate) trait HacheurDeSequence: Hash + Eq + Copy + Send + Sync {
    fn hacher_sequence(seq: &[u8]) -> Self;
}

impl HacheurDeSequence for u64 {
    #[inline(always)]
    fn hacher_sequence(seq: &[u8]) -> Self {
        xxh3_64(seq)
    }
}

impl HacheurDeSequence for u128 {
    #[inline(always)]
    fn hacher_sequence(seq: &[u8]) -> Self {
        xxh3_128(seq)
    }
}

pub(crate) enum TypeDeHachage {
    XXH3_64,
    XXH3_128,
}
