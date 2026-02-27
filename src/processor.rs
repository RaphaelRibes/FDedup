use crate::hasher::{HacheurDeSequence, VerificateurHachage};
use crate::utils::precharger_hachages_existants;
use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::write::GzEncoder;
use needletail::parse_fastx_file;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

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
        if verbeux {
            println!(
                "Option --simulation activée : calcul du taux de duplication sans écriture de fichier."
            );
        }
        let mut lecteur =
            parse_fastx_file(chemin_entree).context("Impossible de lire le fichier d'entrée")?;
        let mut sequences_traitees = 0;
        let mut duplications = 0;

        while let Some(enregistrement) = lecteur.next() {
            let enregistrement_seq = enregistrement.context("Données de séquence invalides")?;

            let hachage = T::hacher_sequence(&enregistrement_seq.seq());
            let est_unique = verificateur.verifier(hachage);

            if !est_unique {
                duplications += 1;
            }

            sequences_traitees += 1;
        }

        return Ok((sequences_traitees, duplications));
    }

    // --- PRÉCHARGEMENT ---
    let chemin_sortie_chemin = Path::new(chemin_sortie);
    let est_gz = chemin_sortie_chemin.extension().and_then(|s| s.to_str()) == Some("gz");

    if forcer {
        if verbeux {
            println!("Option --forcer activée : écrasement de la sortie.");
        }
    } else {
        let (precharges, octets_valides) =
            precharger_hachages_existants(chemin_sortie, &mut verificateur, verbeux)?;
        if precharges > 0 && verbeux {
            println!("{} séquences préchargées.", precharges);
        }

        if chemin_sortie_chemin.exists() {
            let taille_actuelle = fs::metadata(chemin_sortie_chemin)?.len();

            if octets_valides < taille_actuelle {
                if est_gz {
                    bail!(
                        "Le fichier de sortie (.gz) est corrompu et ne peut pas être tronqué. Utilisez --forcer pour recommencer."
                    );
                }

                if verbeux {
                    println!(
                        "Troncature du fichier corrompu de {} à {} octets.",
                        taille_actuelle, octets_valides
                    );
                }
                let fichier = OpenOptions::new().write(true).open(chemin_sortie_chemin)?;
                fichier.set_len(octets_valides)?;
            }
        }
    }

    // --- PRÉPARATION DE L'ÉCRITURE ---
    let fichier_sortie = if chemin_sortie_chemin.exists() && !forcer {
        OpenOptions::new().append(true).open(chemin_sortie_chemin)?
    } else {
        File::create(chemin_sortie_chemin)?
    };

    let ecrivain: Box<dyn Write> = if est_gz {
        Box::new(GzEncoder::new(fichier_sortie, Compression::default()))
    } else {
        Box::new(fichier_sortie)
    };
    let mut sortie_tampon = BufWriter::with_capacity(128 * 1024, ecrivain);

    // --- LECTURE ET PARSING ---
    let mut lecteur =
        parse_fastx_file(chemin_entree).context("Impossible de lire le fichier d'entrée")?;
    let mut sequences_traitees = 0;
    let mut duplications = 0;

    while let Some(enregistrement) = lecteur.next() {
        let enregistrement_seq = enregistrement.context("Données de séquence invalides")?;

        let hachage = T::hacher_sequence(&enregistrement_seq.seq());
        let est_unique = verificateur.verifier(hachage);

        if est_unique {
            enregistrement_seq
                .write(&mut sortie_tampon, None)
                .context("Erreur lors de l'écriture de la séquence")?;
        } else {
            duplications += 1;
        }

        sequences_traitees += 1;
    }

    Ok((sequences_traitees, duplications))
}
