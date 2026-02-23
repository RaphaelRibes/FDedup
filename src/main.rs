mod check_hash;

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;
use std::time::Instant;
use crate::check_hash::HashChecker;

fn lire_chaque_quatrieme_ligne(chemin: &str) -> io::Result<usize> {
    let path = Path::new(chemin);
    let file = File::open(path)?;

    let reader: Box<dyn Read> = if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let buffered_reader = BufReader::new(reader);
    let mut checker = HashChecker::new(1_000_000);

    // 1. Bufferisation de la sortie
    let output_file = File::create("output.fastq")?;
    let mut output = BufWriter::with_capacity(128 * 1024, output_file); // Buffer de 128 KB pour plus d'efficacité

    let mut lines_processed = 0;
    let mut lines = buffered_reader.lines();

    while let Some(line1) = lines.next() {
        let l1 = line1?;
        let l2 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;
        let l3 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;
        let l4 = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "FASTQ tronqué"))??;

        if !checker.check(xxh3_64(l2.as_bytes())) {
            output.write_all(l1.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l2.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l3.as_bytes())?;
            output.write_all(b"\n")?;
            output.write_all(l4.as_bytes())?;
            output.write_all(b"\n")?;
        }

        lines_processed += 1;
    }

    Ok(lines_processed)
}

fn main() {
    let fichier = "data/E94_seq2_R2_001.fastq"; // Fonctionne aussi avec .txt
    let debut_sans = Instant::now();
    if let Ok(inc) = lire_chaque_quatrieme_ligne(fichier) {
        println!("Nombre de lignes traitées : {}", inc);
    }
    let temps_sans = debut_sans.elapsed();

    println!("Temps d'exécution sans multithreading : {:.2?}", temps_sans);
}