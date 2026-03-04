use crate::hasher::{HacheurDeSequence, VerificateurHachage};
use crate::utils::{precharger_hachages_existants, precharger_hachages_existants_paire, preparer_ecrivain};
use anyhow::{Context, Result, bail};
use needletail::parse_fastx_file;
use std::fs::{OpenOptions, self};
use std::io::{BufWriter, Write};
use std::path::Path;

// ==========================================
// MODE SINGLE-END
// ==========================================
pub(crate) fn executer_deduplication<T: HacheurDeSequence + 'static>(
    chemin_entree: &str,
    chemin_sortie: &str,
    forcer: bool,
    verbeux: bool,
    simulation: bool,
    capacite_estimee: usize,
) -> Result<(usize, usize)> {
    let mut verificateur = VerificateurHachage::<T>::nouveau(capacite_estimee);

    if simulation {
        if verbeux { println!("Option --simulation activée : calcul du taux de duplication sans écriture de fichier."); }
        let mut lecteur = parse_fastx_file(chemin_entree).context("Impossible de lire le fichier d'entrée")?;
        let mut sequences_traitees = 0;
        let mut duplications = 0;

        while let Some(enregistrement) = lecteur.next() {
            let enregistrement_seq = enregistrement.context("Données de séquence invalides")?;
            let hachage = T::hacher_sequence(&enregistrement_seq.seq());
            let est_unique = verificateur.verifier(hachage);

            if !est_unique { duplications += 1; }
            sequences_traitees += 1;
        }

        return Ok((sequences_traitees, duplications));
    }

    // --- PRÉCHARGEMENT ---
    let chemin_sortie_chemin = Path::new(chemin_sortie);
    let est_gz = chemin_sortie_chemin.extension().and_then(|s| s.to_str()) == Some("gz");

    if forcer {
        if verbeux { println!("Option --forcer activée : écrasement de la sortie."); }
    } else {
        let (precharges, octets_valides) = precharger_hachages_existants(chemin_sortie, &mut verificateur, verbeux)?;
        if precharges > 0 && verbeux { println!("{} séquences préchargées.", precharges); }

        if chemin_sortie_chemin.exists() {
            let taille_actuelle = fs::metadata(chemin_sortie_chemin)?.len();

            if octets_valides < taille_actuelle {
                if est_gz { bail!("Le fichier de sortie (.gz) est corrompu et ne peut pas être tronqué. Utilisez --forcer pour recommencer."); }
                if verbeux { println!("Troncature du fichier corrompu de {} à {} octets.", taille_actuelle, octets_valides); }
                let fichier = OpenOptions::new().write(true).open(chemin_sortie_chemin)?;
                fichier.set_len(octets_valides)?;
            }
        }
    }

    // --- PRÉPARATION DE L'ÉCRITURE ---
    let ecrivain: Box<dyn Write> = preparer_ecrivain(chemin_sortie_chemin, forcer)?;
    let mut sortie_tampon = BufWriter::with_capacity(128 * 1024, ecrivain);

    // --- LECTURE ET PARSING ---
    let mut lecteur = parse_fastx_file(chemin_entree).context("Impossible de lire le fichier d'entrée")?;
    let mut sequences_traitees = 0;
    let mut duplications = 0;

    while let Some(enregistrement) = lecteur.next() {
        let enregistrement_seq = enregistrement.context("Données de séquence invalides")?;
        let hachage = T::hacher_sequence(&enregistrement_seq.seq());
        let est_unique = verificateur.verifier(hachage);

        if est_unique {
            enregistrement_seq.write(&mut sortie_tampon, None).context("Erreur lors de l'écriture")?;
        } else { duplications += 1; }
        sequences_traitees += 1;
    }

    Ok((sequences_traitees, duplications))
}


// ==========================================
// MODE PAIRED-END
// ==========================================
pub(crate) fn executer_deduplication_paire<T: HacheurDeSequence + 'static>(
    chemin_entree_r1: &str,
    chemin_entree_r2: &str,
    chemin_sortie_r1: &str,
    chemin_sortie_r2: &str,
    forcer: bool,
    verbeux: bool,
    simulation: bool,
    capacite_estimee: usize,
) -> Result<(usize, usize)> {
    let mut verificateur = VerificateurHachage::<T>::nouveau(capacite_estimee);

    // --- LECTURE ET PARSING ---
    let mut lecteur_r1 = parse_fastx_file(chemin_entree_r1).context("Impossible de lire R1")?;
    let mut lecteur_r2 = parse_fastx_file(chemin_entree_r2).context("Impossible de lire R2")?;

    let mut sequences_traitees = 0;
    let mut duplications = 0;

    if simulation {
        if verbeux { println!("Option --simulation activée : calcul du taux de duplication Paired-End."); }

        while let (Some(enreg_r1_res), Some(enreg_r2_res)) = (lecteur_r1.next(), lecteur_r2.next()) {
            let seq_r1 = enreg_r1_res.context("Séquence invalide dans R1")?;
            let seq_r2 = enreg_r2_res.context("Séquence invalide dans R2")?;

            let id_base_r1 = seq_r1.id().split(|&b| b == b' ').next().unwrap_or(seq_r1.id());
            let id_base_r2 = seq_r2.id().split(|&b| b == b' ').next().unwrap_or(seq_r2.id());

            if id_base_r1 != id_base_r2 {
                bail!("Désynchronisation critique détectée ! R1: {}, R2: {}",
                    String::from_utf8_lossy(id_base_r1), String::from_utf8_lossy(id_base_r2));
            }

            let hash_combine = T::hacher_paire(&seq_r1.seq(), &seq_r2.seq());
            if !verificateur.verifier(hash_combine) { duplications += 1; }
            sequences_traitees += 1;
        }

        return Ok((sequences_traitees, duplications));
    }

    // --- PRÉCHARGEMENT ET TRONCATURE SYNCHRONISÉE ---
    let chemin_r1_path = Path::new(chemin_sortie_r1);
    let chemin_r2_path = Path::new(chemin_sortie_r2);

    let est_gz_r1 = chemin_r1_path.extension().and_then(|s| s.to_str()) == Some("gz");
    let est_gz_r2 = chemin_r2_path.extension().and_then(|s| s.to_str()) == Some("gz");

    if forcer {
        if verbeux { println!("Option --forcer activée : écrasement des sorties Paired-End."); }
    } else {
        let (precharges, octets_r1, octets_r2) = precharger_hachages_existants_paire(
            chemin_sortie_r1, chemin_sortie_r2, &mut verificateur, verbeux
        )?;

        if precharges > 0 && verbeux {
            println!("{} paires préchargées et synchronisées.", precharges);
        }

        // Troncature sécurisée de R1
        if chemin_r1_path.exists() {
            let taille_actuelle_r1 = fs::metadata(chemin_r1_path)?.len();
            if octets_r1 < taille_actuelle_r1 {
                if est_gz_r1 { bail!("Le fichier R1 (.gz) est désynchronisé et ne peut être tronqué. Utilisez --forcer."); }
                if verbeux { println!("Troncature de R1 pour resynchronisation ({} à {} octets).", taille_actuelle_r1, octets_r1); }
                let fichier = OpenOptions::new().write(true).open(chemin_r1_path)?;
                fichier.set_len(octets_r1)?;
            }
        }

        // Troncature sécurisée de R2
        if chemin_r2_path.exists() {
            let taille_actuelle_r2 = fs::metadata(chemin_r2_path)?.len();
            if octets_r2 < taille_actuelle_r2 {
                if est_gz_r2 { bail!("Le fichier R2 (.gz) est désynchronisé et ne peut être tronqué. Utilisez --forcer."); }
                if verbeux { println!("Troncature de R2 pour resynchronisation ({} à {} octets).", taille_actuelle_r2, octets_r2); }
                let fichier = OpenOptions::new().write(true).open(chemin_r2_path)?;
                fichier.set_len(octets_r2)?;
            }
        }
    }

    // --- PRÉPARATION DE L'ÉCRITURE ---
    let ecrivain_r1: Box<dyn Write> = preparer_ecrivain(chemin_r1_path, forcer)?;
    let ecrivain_r2: Box<dyn Write> = preparer_ecrivain(chemin_r2_path, forcer)?;

    let mut sortie_tampon_r1 = BufWriter::with_capacity(128 * 1024, ecrivain_r1);
    let mut sortie_tampon_r2 = BufWriter::with_capacity(128 * 1024, ecrivain_r2);

    // --- BOUCLE PRINCIPALE D'ÉCRITURE SYNCHRONISÉE ---
    while let (Some(enreg_r1_res), Some(enreg_r2_res)) = (lecteur_r1.next(), lecteur_r2.next()) {
        let seq_r1 = enreg_r1_res.context("Séquence invalide dans R1")?;
        let seq_r2 = enreg_r2_res.context("Séquence invalide dans R2")?;

        let id_base_r1 = seq_r1.id().split(|&b| b == b' ').next().unwrap_or(seq_r1.id());
        let id_base_r2 = seq_r2.id().split(|&b| b == b' ').next().unwrap_or(seq_r2.id());

        if id_base_r1 != id_base_r2 {
            bail!("Désynchronisation critique détectée à la paire n°{} ! R1: {}, R2: {}",
                sequences_traitees + 1, String::from_utf8_lossy(id_base_r1), String::from_utf8_lossy(id_base_r2));
        }

        let hash_combine = T::hacher_paire(&seq_r1.seq(), &seq_r2.seq());
        let est_unique = verificateur.verifier(hash_combine);

        if est_unique {
            seq_r1.write(&mut sortie_tampon_r1, None).context("Erreur écriture R1")?;
            seq_r2.write(&mut sortie_tampon_r2, None).context("Erreur écriture R2")?;
        } else {
            duplications += 1;
        }

        sequences_traitees += 1;
    }

    if lecteur_r1.next().is_some() || lecteur_r2.next().is_some() {
        bail!("Désynchronisation détectée à la fin : un fichier contient plus de lectures que l'autre !");
    }

    Ok((sequences_traitees, duplications))
}