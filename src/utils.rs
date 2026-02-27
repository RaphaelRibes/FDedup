use anyhow::{Context, Result};
use needletail::parse_fastx_file;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

use crate::hasher::{HacheurDeSequence, TypeDeHachage, VerificateurHachage};

pub fn recuperer_methode_de_hachage(taille: usize, seuil: f64) -> TypeDeHachage {
    if (2f64 * 2.0f64.powi(64) * seuil).sqrt() < taille as f64 {
        TypeDeHachage::XXH3_128
    } else {
        TypeDeHachage::XXH3_64
    }
}

pub fn estimer_capacite_sequences<P: AsRef<Path>>(chemin: P) -> Result<usize> {
    let chemin_precis = chemin.as_ref();
    if !chemin_precis.exists() {
        return Ok(0);
    }

    let metadonnees = fs::metadata(chemin_precis)?;
    let taille_fichier_octets = metadonnees.len();
    let est_gz = chemin_precis.extension().and_then(|s| s.to_str()) == Some("gz");

    let capacite_estimee = if est_gz {
        (taille_fichier_octets / 80) as usize
    } else {
        (taille_fichier_octets / 350) as usize
    };

    Ok(capacite_estimee)
}

pub fn precharger_hachages_existants<T: HacheurDeSequence>(
    chemin: &str,
    verificateur: &mut VerificateurHachage<T>,
    verbeux: bool,
) -> Result<(usize, u64)> {
    if !Path::new(chemin).exists() {
        return Ok((0, 0));
    }

    if verbeux {
        println!("Préchargement des séquences depuis la sortie existante...");
    }

    let mut lecteur = parse_fastx_file(chemin)
        .context("Erreur lors de l'ouverture du fichier de préchargement")?;
    let mut compte = 0;
    let mut octets_valides = CompteurDOctets(0);

    while let Some(enregistrement_resultat) = lecteur.next() {
        match enregistrement_resultat {
            Ok(enregistrement) => {
                let hachage = T::hacher_sequence(&enregistrement.seq());
                verificateur.verifier(hachage);

                compte += 1;
                let _ = enregistrement.write(&mut octets_valides, None);
            }
            Err(e) => {
                if verbeux {
                    eprintln!("Séquence incomplète détectée à la fin du fichier ({}).", e);
                    eprintln!("Calcul du point de troncature de sécurité...");
                }
                break;
            }
        }
    }

    Ok((compte, octets_valides.0))
}

struct CompteurDOctets(u64);

impl Write for CompteurDOctets {
    fn write(&mut self, tampon: &[u8]) -> io::Result<usize> {
        self.0 += tampon.len() as u64;
        Ok(tampon.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
