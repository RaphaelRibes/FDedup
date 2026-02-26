use std::{fs, io};
use std::io::Write;
use std::path::Path;
use anyhow::{Result, Context};
use needletail::parse_fastx_file;

use crate::hasher::{HashChecker, HashType, SequenceHasher};

pub fn récupérer_la_méthode_de_hachage
(
    size: usize,
    threshold: f64
)
    -> HashType
{
    if (2f64 * 2.0f64.powi(64) * threshold).sqrt() < size as f64 {
        HashType::XXH3_128
    } else {
        HashType::XXH3_64
    }
}

pub fn estimer_capacite_sequences<P: AsRef<Path>>(chemin: P) -> Result<usize> {
    let path = chemin.as_ref();
    if !path.exists() {
        return Ok(0);
    }

    let metadata = fs::metadata(path)?;
    let file_size_bytes = metadata.len();
    let is_gz = path.extension().and_then(|s| s.to_str()) == Some("gz");

    let estimated_capacity = if is_gz {
        (file_size_bytes / 80) as usize
    } else {
        (file_size_bytes / 350) as usize
    };

    Ok(estimated_capacity)
}

pub fn precharger_hashes_existants<T: SequenceHasher>
(
    chemin: &str,
    checker: &mut HashChecker<T>,
    verbose: bool
)
    -> Result<(usize, u64)>
{
    if !Path::new(chemin).exists() {
        return Ok((0, 0));
    }

    if verbose {
        println!("Préchargement des séquences depuis l'output existant...");
    }

    let mut reader = parse_fastx_file(chemin).context("Erreur lors de l'ouverture du fichier de préchargement")?;
    let mut count = 0;
    let mut valid_bytes = ByteCounter(0);

    while let Some(record) = reader.next() {
        match record {
            Ok(seqrec) => {
                let hash = T::hash_seq(&seqrec.seq());
                checker.check(hash);

                count += 1;
                let _ = seqrec.write(&mut valid_bytes, None);
            }
            Err(e) => {
                if verbose {
                    eprintln!("Séquence incomplète détectée à la fin du fichier ({}).", e);
                    eprintln!("Calcul du point de troncature de sécurité...");
                }
                break;
            }
        }
    }

    Ok((count, valid_bytes.0))
}

struct ByteCounter(u64);

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}