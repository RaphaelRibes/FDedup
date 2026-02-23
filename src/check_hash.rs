use rustc_hash::FxHashSet;

pub struct HashChecker {
    // FxHashSet remplace le HashSet standard pour des performances maximales
    // sur des clefs entières sans surcoût cryptographique.
    memoire: FxHashSet<u64>,
}

impl HashChecker {
    /// Initialise la structure en pré-allouant la mémoire.
    pub fn new(capacite_estimee: usize) -> Self {
        Self {
            // La pré-allocation évite à Rust de redimensionner la mémoire
            // dynamiquement lors de l'insertion des 1.5 million de clefs.
            memoire: FxHashSet::with_capacity_and_hasher(
                capacite_estimee,
                Default::default()
            ),
        }
    }

    /// Vérifie si la clef est en mémoire.
    /// Retourne `true` si trouvée, sinon la sauvegarde et retourne `false`.
    pub fn check(&mut self, hash: u64) -> bool {
        // `insert` sauvegarde la clef et retourne `true` si elle était absente.
        // L'opérateur `!` inverse le booléen pour coller à votre besoin.
        !self.memoire.insert(hash)
    }
}
