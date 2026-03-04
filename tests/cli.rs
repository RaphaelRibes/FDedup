use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

// --------------------------------------------------------
// UTILITAIRES DE GÉNÉRATION DE FICHIERS DE TEST
// --------------------------------------------------------

fn generer_fastq(
    dir: &assert_fs::TempDir,
    nom_fichier: &str,
    nb_uniques: usize,
    nb_duplications: usize,
) -> assert_fs::fixture::ChildPath {
    let fichier = dir.child(nom_fichier);
    let mut contenu = String::new();

    let nucleotides = [b'A', b'C', b'G', b'T', b'N'];

    let generer_sequence_complexe = |index: usize| -> String {
        let mut seq = Vec::with_capacity(150);
        let mut temp_index = index;

        for _ in 0..150 {
            seq.push(nucleotides[temp_index % nucleotides.len()]);
            temp_index /= nucleotides.len();
        }

        while seq.len() < 150 {
            seq.push(nucleotides[(index + seq.len()) % nucleotides.len()]);
        }

        String::from_utf8(seq).expect("Séquence UTF-8 invalide")
    };

    // 1. Séquences uniques
    for i in 0..nb_uniques {
        let header = format!("@A00123:456:HFWV2DSXX:1:1101:1000:{} 1:N:0:ATGC", 1000 + i);
        contenu.push_str(&header);
        contenu.push('\n');
        contenu.push_str(&generer_sequence_complexe(i));
        contenu.push_str("\n+\n");
        let qual = (0..150).map(|j| if (i + j) % 10 == 0 { ',' } else { 'F' }).collect::<String>();
        contenu.push_str(&qual);
        contenu.push('\n');
    }

    // 2. Duplicats
    for i in 0..nb_duplications {
        let header = format!("@A00123:456:HFWV2DSXX:1:1101:5000:{} 1:N:0:ATGC", 1000 + i);
        contenu.push_str(&header);
        contenu.push('\n');
        contenu.push_str(&generer_sequence_complexe(i));
        contenu.push_str("\n+\n");
        let qual = (0..150).map(|j| if (i + j) % 10 == 0 { ',' } else { 'F' }).collect::<String>();
        contenu.push_str(&qual);
        contenu.push('\n');
    }

    fichier.write_str(&contenu).unwrap();
    fichier
}

fn generer_fastq_paire(
    dir: &assert_fs::TempDir,
    nom_r1: &str,
    nom_r2: &str,
    nb_uniques: usize,
    nb_duplications: usize,
) -> (assert_fs::fixture::ChildPath, assert_fs::fixture::ChildPath) {
    let fichier_r1 = dir.child(nom_r1);
    let fichier_r2 = dir.child(nom_r2);
    let mut contenu_r1 = String::new();
    let mut contenu_r2 = String::new();

    let nucleotides = [b'A', b'C', b'G', b'T', b'N'];

    let generer_sequence_complexe = |index: usize, is_r2: bool| -> String {
        let mut seq = Vec::with_capacity(150);
        let mut temp_index = if is_r2 { index + 9999 } else { index }; // Séquence différente pour R2

        for _ in 0..150 {
            seq.push(nucleotides[temp_index % nucleotides.len()]);
            temp_index /= nucleotides.len();
        }

        while seq.len() < 150 {
            seq.push(nucleotides[(index + seq.len()) % nucleotides.len()]);
        }
        String::from_utf8(seq).expect("Séquence UTF-8 invalide")
    };

    // 1. Paires Uniques
    for i in 0..nb_uniques {
        let base_id = format!("@A00123:456:HFWV2DSXX:1:1101:1000:{}", 1000 + i);

        contenu_r1.push_str(&format!("{} 1:N:0:ATGC\n{}\n+\n{}\n", base_id, generer_sequence_complexe(i, false), "F".repeat(150)));
        contenu_r2.push_str(&format!("{} 2:N:0:ATGC\n{}\n+\n{}\n", base_id, generer_sequence_complexe(i, true), "F".repeat(150)));
    }

    // 2. Paires Dupliquées (Vrais PCR duplicates : R1 ET R2 identiques)
    for i in 0..nb_duplications {
        let base_id = format!("@A00123:456:HFWV2DSXX:1:1101:5000:{}", 1000 + i);

        contenu_r1.push_str(&format!("{} 1:N:0:ATGC\n{}\n+\n{}\n", base_id, generer_sequence_complexe(i, false), "F".repeat(150)));
        contenu_r2.push_str(&format!("{} 2:N:0:ATGC\n{}\n+\n{}\n", base_id, generer_sequence_complexe(i, true), "F".repeat(150)));
    }

    fichier_r1.write_str(&contenu_r1).unwrap();
    fichier_r2.write_str(&contenu_r2).unwrap();

    (fichier_r1, fichier_r2)
}


// --------------------------------------------------------
// TESTS SINGLE-END
// --------------------------------------------------------

#[test]
fn test_fichier_entree_inexistant() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.current_dir(temp_dir.path())
        .arg("-1")
        .arg("fichier_fantome.fastq")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Impossible de lire le fichier d'entrée"));
}

#[test]
fn test_fichier_entree_vide() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let fichier_vide = temp_dir.child("vide.fastq");
    fichier_vide.touch().unwrap();

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1")
        .arg(fichier_vide.path())
        .arg("-s") // simulation
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read the first two bytes. Is the file empty?"));
}

#[test]
fn test_fichier_entree_mauvais_format() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let fichier_texte = temp_dir.child("mauvais_format.txt");
    fichier_texte.write_str("Ceci n'est pas un fichier FASTQ mais un fichier texte normal\nAvec plusieurs lignes.").unwrap();

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1")
        .arg(fichier_texte.path())
        .arg("-s")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Données de séquence invalides")
            .or(predicate::str::contains("Impossible de lire le fichier")));
}

#[test]
fn test_fichier_sortie_sans_permissions_ou_introuvable() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let fichier_entree = generer_fastq(&temp_dir, "entree.fastq", 10, 0);

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1")
        .arg(fichier_entree.path())
        .arg("-o")
        .arg("/chemin/vers/un/dossier/qui/n/existe/pas/sortie.fastq")
        .assert()
        .failure();
}

#[test]
fn test_fichier_sortie_gz_corrompu() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let fichier_entree = generer_fastq(&temp_dir, "entree.fastq", 10, 0);

    let fichier_sortie_corrompu = temp_dir.child("sortie_corrompue.fastq.gz");
    fichier_sortie_corrompu.write_str("Ceci n'est pas un fichier compressé gzip valide.").unwrap();

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1")
        .arg(fichier_entree.path())
        .arg("-o")
        .arg(fichier_sortie_corrompu.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Erreur lors de l'ouverture du fichier de préchargement"));
}

#[test]
fn test_taux_duplication_10_et_50_pourcents_single() {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // 100 séquences uniques
    let seq_100_uniques = generer_fastq(&temp_dir, "100_uniques.fastq", 100, 0);
    cargo::cargo_bin_cmd!("fdedup")
        .arg("-1").arg(seq_100_uniques.path()).arg("-s").arg("-v")
        .assert().success()
        .stdout(predicate::str::contains("Fragments traités : 100"))
        .stdout(predicate::str::contains("Doublons supprimés : 0.00%"));

    // 110 séquences (100 uniques + 10 dupliquées)
    let seq_110 = generer_fastq(&temp_dir, "110_seq.fastq", 100, 10);
    cargo::cargo_bin_cmd!("fdedup")
        .arg("-1").arg(seq_110.path()).arg("-s").arg("-v")
        .assert().success()
        .stdout(predicate::str::contains("Fragments traités : 110"))
        .stdout(predicate::str::contains("Doublons supprimés : 9.09%"));
}

// --------------------------------------------------------
// NOUVEAUX TESTS PAIRED-END
// --------------------------------------------------------

#[test]
fn test_paire_taux_duplication() {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // 100 Paires uniques + 50 Paires dupliquées = 150 paires totales
    let (r1, r2) = generer_fastq_paire(&temp_dir, "R1.fastq", "R2.fastq", 100, 50);

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1").arg(r1.path())
        .arg("-2").arg(r2.path())
        .arg("-p").arg("dummy_r2.fastq") // <-- L'argument de sortie obligatoire ajouté ici !
        .arg("-s") // Simulation
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("Paired-End"))
        .stdout(predicate::str::contains("Fragments traités : 150"))
        .stdout(predicate::str::contains("Doublons supprimés : 33.33%"));
}

#[test]
fn test_paire_erreur_sortie_r2_manquante() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let (r1, r2) = generer_fastq_paire(&temp_dir, "R1.fastq", "R2.fastq", 10, 0);

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    // On passe R1, R2, et la sortie R1 (-o), mais on oublie délibérément la sortie R2 (-p)
    cmd.arg("-1").arg(r1.path())
        .arg("-2").arg(r2.path())
        .arg("-o").arg("sortie_r1_test.fastq")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Erreur critique : L'argument --sortie-r2 (-p) est obligatoire"));
}

#[test]
fn test_paire_desynchronisee_bloquee() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let (r1, r2) = generer_fastq_paire(&temp_dir, "R1_desync.fastq", "R2_desync.fastq", 10, 0);

    // On simule une désynchronisation en ajoutant manuellement une ligne "orpheline" au début de R2
    let mut contenu_r2_corrompu = String::from("@A00123:456:HFWV2DSXX:1:1101:9999:9999 2:N:0:ATGC\nATGC\n+\nFFFF\n");
    contenu_r2_corrompu.push_str(&std::fs::read_to_string(r2.path()).unwrap());
    r2.write_str(&contenu_r2_corrompu).unwrap();

    let mut cmd = cargo::cargo_bin_cmd!("fdedup");
    cmd.arg("-1").arg(r1.path())
        .arg("-2").arg(r2.path())
        .arg("-p").arg("dummy_r2.fastq") // <-- L'argument de sortie obligatoire ajouté ici !
        .arg("-s") // Simulation
        .assert()
        .failure()
        .stderr(predicate::str::contains("Désynchronisation critique détectée"));
}