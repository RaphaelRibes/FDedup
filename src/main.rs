mod cli;
mod hasher;
mod processor;
mod utils;

use crate::hasher::TypeDeHachage;
use crate::processor::executer_deduplication;
use crate::utils::estimer_capacite_sequences;
use crate::utils::recuperer_methode_de_hachage;
use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, ModeHachage};
use std::time::Instant;

fn distribuer(
    chemin_entree: &str,
    chemin_sortie: &str,
    capacite_estimee: usize,
    forcer: bool,
    verbeux: bool,
    simulation: bool,
    type_de_hachage: TypeDeHachage,
) -> Result<(usize, usize)> {
    match type_de_hachage {
        TypeDeHachage::XXH3_64 => {
            if verbeux {
                println!("Mode : Hachage 64 bits");
            }
            executer_deduplication::<u64>(
                chemin_entree,
                chemin_sortie,
                forcer,
                verbeux,
                simulation,
                capacite_estimee,
            )
        }
        TypeDeHachage::XXH3_128 => {
            if verbeux {
                println!("Mode : Hachage 128 bits");
            }
            executer_deduplication::<u128>(
                chemin_entree,
                chemin_sortie,
                forcer,
                verbeux,
                simulation,
                capacite_estimee,
            )
        }
    }
}

fn main() -> Result<()> {
    let arguments = Cli::parse();

    if arguments.hachage.is_some() && arguments.seuil != 0.01 {
        eprintln!(
            "Avertissement : --hachage spécifié, le seuil de sélection automatique ({}) est ignoré.",
            arguments.seuil
        );
    }

    if arguments.verbeux {
        println!("Fichier d'entrée : {}", arguments.entree);
        println!("Fichier de sortie : {}", arguments.sortie);
    }

    let cap_entree = estimer_capacite_sequences(&arguments.entree)
        .context("Fichier d'entrée introuvable ou inaccessible")?;

    let cap_sortie = if arguments.forcer {
        0
    } else {
        estimer_capacite_sequences(&arguments.sortie).unwrap_or(0)
    };
    let capacite_totale = cap_entree + cap_sortie;

    let type_de_hachage_selectionne = match arguments.hachage {
        Some(ModeHachage::Bit64) => TypeDeHachage::XXH3_64,
        Some(ModeHachage::Bit128) => TypeDeHachage::XXH3_128,
        None => recuperer_methode_de_hachage(capacite_totale, arguments.seuil),
    };

    if arguments.verbeux {
        println!("Capacité totale estimée : {} séquences", capacite_totale);
    }

    let debut = Instant::now();

    let (traitees, duplications) = distribuer(
        &arguments.entree,
        &arguments.sortie,
        capacite_totale,
        arguments.forcer,
        arguments.verbeux,
        arguments.simulation,
        type_de_hachage_selectionne,
    )?;

    if arguments.verbeux {
        println!(
            "Traitées : {}\nDuplication : {:.2}%",
            traitees,
            if traitees > 0 {
                duplications as f64 / traitees as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("Temps d'exécution total : {:.2?}", debut.elapsed());
    }

    Ok(())
}
