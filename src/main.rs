mod check_hash;

use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;
use std::time::Instant;
use crate::check_hash::HashChecker;
use std::fs;

fn estimer_capacite_sequences(chemin: &str) -> io::Result<usize> {
    let path = Path::new(chemin);
    let metadata = fs::metadata(path)?;
    let file_size_bytes = metadata.len();

    let is_gz = path.extension().and_then(|s| s.to_str()) == Some("gz");

    // Diviseurs heuristiques : ~80 octets/séquence pour un .gz, ~350 octets/séquence en clair.
    let estimated_capacity = if is_gz {
        (file_size_bytes / 80) as usize
    } else {
        (file_size_bytes / 350) as usize
    };

    Ok(estimated_capacity)
}

fn lire_chaque_quatrieme_ligne(chemin_entree: &str, chemin_sortie: &str) -> io::Result<usize> {
    // --- LECTURE ---
    let path_in = Path::new(chemin_entree);
    let file_in = File::open(path_in)?;

    let reader: Box<dyn Read> = if path_in.extension().and_then(|s| s.to_str()) == Some("gz") {
        Box::new(MultiGzDecoder::new(file_in))
    } else {
        Box::new(file_in)
    };
    let buffered_reader = BufReader::new(reader);

    // --- ÉCRITURE ---
    let path_out = Path::new(chemin_sortie);
    let file_out = File::create(path_out)?;

    // Détection de l'extension pour compresser ou non la sortie
    let writer: Box<dyn Write> = if path_out.extension().and_then(|s| s.to_str()) == Some("gz") {
        // On utilise la compression par défaut, mais tu peux utiliser Compression::fast() si besoin
        Box::new(GzEncoder::new(file_out, Compression::default()))
    } else {
        Box::new(file_out)
    };

    let mut output = BufWriter::with_capacity(128 * 1024, writer);

    // --- TRAITEMENT ---
    let estimated_capacity = estimer_capacite_sequences(chemin_entree)?;
    println!("Capacité estimée pour le HashChecker : {} séquences", estimated_capacity);
    let mut checker = HashChecker::new(estimated_capacity);
    let mut lines_processed = 0;
    let mut lines = buffered_reader.lines();

    while let Some(line1) = lines.next() {
        let l1 = line1?;
        let l2 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;

        if !checker.check(xxh3_64(l2.as_bytes())) {
            let l3 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;
            let l4 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;

            output.write_all(l1.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l2.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l3.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l4.as_bytes())?;
            output.write_all(b"\n")?;
        } else {
            lines.next(); // Skip line 3
            lines.next(); // Skip line 4
        }

        lines_processed += 1;
    }

    Ok(lines_processed)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <fichier_entree> [fichier_sortie]", args[0]);
        std::process::exit(1);
    }

    let fichier_entree = &args[1];

    let fichier_sortie = if args.len() >= 3 {
        &args[2]
    } else {
        "output.fastq.gz"
    };

    println!("Fichier d'entrée : {}", fichier_entree);
    println!("Fichier de sortie : {}", fichier_sortie);

    let debut_sans = Instant::now();
    match lire_chaque_quatrieme_ligne(fichier_entree, fichier_sortie) {
        Ok(inc) => println!("Nombre de séquences traitées : {}", inc),
        Err(e) => eprintln!("Erreur pendant le traitement : {:?}", e),
    }
    let temps_sans = debut_sans.elapsed();

    println!("Temps d'exécution sans multithreading : {:.2?}", temps_sans);
}