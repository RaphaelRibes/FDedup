mod check_hash;

use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write, IoSlice};
use std::path::Path;
use std::time::Instant;
use xxhash_rust::xxh3::xxh3_64;

use crate::check_hash::HashChecker;

fn estimer_capacite_sequences(chemin: &str) -> io::Result<usize> {
    let path = Path::new(chemin);
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

fn get_reader(chemin: &str) -> io::Result<Box<dyn Read>> {
    let path = Path::new(chemin);
    let file = File::open(path)?;
    if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        Ok(Box::new(MultiGzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
}

fn get_l2(l2: &[u8]) -> &[u8] {
    let l2_len = l2.len();
    if l2_len > 0 && l2[l2_len - 1] == b'\n' {
        if l2_len > 1 && l2[l2_len - 2] == b'\r' {
            &l2[..l2_len - 2]
        } else {
            &l2[..l2_len - 1]
        }
    } else {
        &l2[..]
    }
}

fn precharger_hashes_existants(chemin: &str, checker: &mut HashChecker, verbose: bool) -> io::Result<usize> {
    let path = Path::new(chemin);
    if !path.exists() {
        return Ok(0);
    }

    if verbose {
        println!("Préchargement des séquences depuis l'output existant...");
    }

    let reader: Box<dyn Read> = get_reader(chemin)?;
    let mut buffered_reader = BufReader::new(reader);

    let mut l2 = Vec::new();
    let mut trash = Vec::new();
    let mut count = 0;

    loop {
        trash.clear();
        if buffered_reader.read_until(b'\n', &mut trash)? == 0 { break; }

        l2.clear();
        if buffered_reader.read_until(b'\n', &mut l2)? == 0 { break; }

        let l2_content = get_l2(&l2);

        checker.check(xxh3_64(l2_content));
        count += 1;

        trash.clear();
        if buffered_reader.read_until(b'\n', &mut trash)? == 0 { break; }
        trash.clear();
        if buffered_reader.read_until(b'\n', &mut trash)? == 0 { break; }
    }

    Ok(count)
}

fn lire_chaque_quatrieme_ligne(chemin_entree: &str, chemin_sortie: &str, force: bool, verbose: bool) -> io::Result<(usize, usize)> {
    // --- ESTIMATION ET INITIALISATION DU CHECKER ---
    let cap_entree = estimer_capacite_sequences(chemin_entree)?;
    let cap_sortie = if force { 0 } else { estimer_capacite_sequences(chemin_sortie).unwrap_or(0) };
    let estimated_capacity = cap_entree + cap_sortie;
    let mut checker = HashChecker::new(estimated_capacity);

    if verbose {
        println!("Capacité estimée totale pour le HashChecker : {} séquences", estimated_capacity);
    }


    // --- PRÉCHARGEMENT ---
    if force {
        if verbose { println!("Option --force activée : le fichier de sortie existant sera écrasé."); }
    } else {
        let preloaded = precharger_hashes_existants(chemin_sortie, &mut checker, verbose)?;
        if preloaded > 0 && verbose { println!("{} séquences préchargées depuis le fichier de sortie.", preloaded); }
    }


    // --- LECTURE ENTRÉE ---
    let reader: Box<dyn Read> = get_reader(chemin_entree)?;
    let mut buffered_reader = BufReader::new(reader);


    // --- ÉCRITURE SORTIE ---
    let path_out = Path::new(chemin_sortie);


    // MODE APPEND OU OVERWRITE selon la présence du flag `force`
    let file_out = if path_out.exists() && !force {
        OpenOptions::new().append(true).open(path_out)?
    } else {
        File::create(path_out)?
    };

    let writer: Box<dyn Write> = if path_out.extension().and_then(|s| s.to_str()) == Some("gz") {
        Box::new(GzEncoder::new(file_out, Compression::default()))
    } else {
        Box::new(file_out)
    };

    let mut output = BufWriter::with_capacity(128 * 1024, writer);


    // --- TRAITEMENT ---
    let mut lines_processed = 0;
    let mut dups = 0;

    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    let mut l3 = Vec::new();
    let mut l4 = Vec::new();

    loop {
        l1.clear();
        l2.clear();

        if buffered_reader.read_until(b'\n', &mut l1)? == 0 {
            break;
        }

        if buffered_reader.read_until(b'\n', &mut l2)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"));
        }

        let l2_content = get_l2(&l2);

        let is_unique = !checker.check(xxh3_64(l2_content));

        l3.clear();
        l4.clear();
        if buffered_reader.read_until(b'\n', &mut l3)? == 0 ||
            buffered_reader.read_until(b'\n', &mut l4)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"));
        }

        if is_unique {
            let bufs = [
                IoSlice::new(&l1),
                IoSlice::new(&l2),
                IoSlice::new(&l3),
                IoSlice::new(&l4),
            ];
            output.write_vectored(&bufs)?;
        } else {
            dups += 1;
        }

        lines_processed += 1;
    }

    Ok((lines_processed, dups))
}

fn main() {
    let mut args = env::args();
    let executable_name = args.next().unwrap_or_else(|| "programme".to_string());

    let mut force = false;
    let mut verbose = false;
    let mut positional_args = Vec::new();

    // Analyse des arguments
    for arg in args {
        if arg == "--force" {
            force = true;
        } else if arg == "--verbose" || arg == "-v" {
            verbose = true;
        } else {
            positional_args.push(arg);
        }
    }

    if positional_args.is_empty() {
        eprintln!("Usage: {} <fichier_entree> [fichier_sortie] [--force] [--verbose|-v]", executable_name);
        std::process::exit(1);
    }

    let fichier_entree = &positional_args[0];

    let fichier_sortie = if positional_args.len() >= 2 {
        &positional_args[1]
    } else {
        "output.fastq.gz"
    };

    if verbose {
        println!("Fichier d'entrée : {}", fichier_entree);
        println!("Fichier de sortie : {}", fichier_sortie);
    }

    let debut_sans = Instant::now();
    match lire_chaque_quatrieme_ligne(fichier_entree, fichier_sortie, force, verbose) {
        Ok((inc, dup)) => {
            if verbose {
                println!(
                    "Nombre de séquences de l'entrée traitées : {}\n%tage de duplication dans l'entrée : {:.2}%",
                    inc,
                    if inc > 0 { dup as f64 / inc as f64 * 100.0 } else { 0.0 }
                );
            }
        },
        Err(e) => {
            match e.kind() {
                io::ErrorKind::UnexpectedEof => eprintln!("Erreur : Le fichier FASTQ semble être tronqué."),
                _ => eprintln!("Erreur lors du traitement : {}", e),
                }
        },
    }
    let temps_sans = debut_sans.elapsed();

    if verbose {
        println!("Temps d'exécution total : {:.2?}", temps_sans);
    }
}