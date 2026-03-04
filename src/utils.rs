use anyhow::{Context, Result};
use needletail::parse_fastx_file;
use std::io::Write;
use std::path::Path;
use std::{fs, io};
use std::fs::{File, OpenOptions};
use flate2::write::GzEncoder;
use flate2::Compression;
use crate::hasher::{HacheurDeSequence, TypeDeHachage, VerificateurHachage};

pub fn recuperer_methode_de_hachage(taille: usize, seuil: f64) -> TypeDeHachage {
    if (2f64 * 2.0f64.powi(64) * seuil).sqrt() < taille as f64 {
        TypeDeHachage::XXH3_128
    } else {
        TypeDeHachage::XXH3_64
    }
}

pub fn preparer_ecrivain(chemin: &Path, forcer: bool) -> Result<Box<dyn Write>> {
    let est_gz = chemin.extension().and_then(|s| s.to_str()) == Some("gz");

    let fichier = if chemin.exists() && !forcer {
        OpenOptions::new().append(true).open(chemin)
            .with_context(|| format!("Impossible d'ouvrir le fichier en ajout : {:?}", chemin))?
    } else {
        File::create(chemin)
            .with_context(|| format!("Impossible de créer le fichier : {:?}", chemin))?
    };

    if est_gz {
        Ok(Box::new(GzEncoder::new(fichier, Compression::default())))
    } else {
        Ok(Box::new(fichier))
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

pub fn precharger_hachages_existants_paire<T: HacheurDeSequence>(
    chemin_r1: &str,
    chemin_r2: &str,
    verificateur: &mut VerificateurHachage<T>,
    verbeux: bool,
) -> Result<(usize, u64, u64)> {
    let path_r1 = Path::new(chemin_r1);
    let path_r2 = Path::new(chemin_r2);

    if !path_r1.exists() || !path_r2.exists() {
        return Ok((0, 0, 0));
    }

    if verbeux {
        println!("Préchargement et synchronisation des paires depuis les sorties existantes...");
    }

    let mut lecteur_r1 = parse_fastx_file(chemin_r1).context("Erreur ouverture préchargement R1")?;
    let mut lecteur_r2 = parse_fastx_file(chemin_r2).context("Erreur ouverture préchargement R2")?;

    let mut compte = 0;
    let mut octets_valides_r1 = CompteurDOctets(0);
    let mut octets_valides_r2 = CompteurDOctets(0);

    while let (Some(enreg_r1_res), Some(enreg_r2_res)) = (lecteur_r1.next(), lecteur_r2.next()) {
        match (enreg_r1_res, enreg_r2_res) {
            (Ok(enreg_r1), Ok(enreg_r2)) => {
                let hash_combine = T::hacher_paire(&enreg_r1.seq(), &enreg_r2.seq());
                verificateur.verifier(hash_combine);

                compte += 1;
                let _ = enreg_r1.write(&mut octets_valides_r1, None);
                let _ = enreg_r2.write(&mut octets_valides_r2, None);
            }
            _ => {
                if verbeux {
                    eprintln!("Séquence incomplète ou désynchronisation détectée en fin de fichier.");
                    eprintln!("Calcul des points de troncature de sécurité pour R1 et R2...");
                }
                break;
            }
        }
    }

    Ok((compte, octets_valides_r1.0, octets_valides_r2.0))
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