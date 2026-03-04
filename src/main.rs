mod cli;
mod hasher;
mod processor;
mod utils;

use crate::hasher::TypeDeHachage;
use crate::processor::{executer_deduplication, executer_deduplication_paire};
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
)
    -> Result<(usize, usize)>
{
    match type_de_hachage {
        TypeDeHachage::XXH3_64 => {
            if verbeux {
                println!("Mode Single-End : Hachage 64 bits");
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
                println!("Mode Single-End : Hachage 128 bits");
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

// Nouvelle fonction de distribution pour le mode Paired-End
fn distribuer_paire(
    chemin_entree_r1: &str,
    chemin_entree_r2: &str,
    chemin_sortie_r1: &str,
    chemin_sortie_r2: &str,
    capacite_estimee: usize,
    forcer: bool,
    verbeux: bool,
    simulation: bool,
    type_de_hachage: TypeDeHachage,
)
    -> Result<(usize, usize)>
{
    match type_de_hachage {
        TypeDeHachage::XXH3_64 => {
            if verbeux {
                println!("Mode Paired-End : Hachage 64 bits combiné");
            }
            executer_deduplication_paire::<u64>(
                chemin_entree_r1,
                chemin_entree_r2,
                chemin_sortie_r1,
                chemin_sortie_r2,
                forcer,
                verbeux,
                simulation,
                capacite_estimee,
            )
        }
        TypeDeHachage::XXH3_128 => {
            if verbeux {
                println!("Mode Paired-End : Hachage 128 bits combiné");
            }
            executer_deduplication_paire::<u128>(
                chemin_entree_r1,
                chemin_entree_r2,
                chemin_sortie_r1,
                chemin_sortie_r2,
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
        println!("Fichier d'entrée principal (R1) : {}", arguments.entree);
        println!("Fichier de sortie principal (R1) : {}", arguments.sortie);
    }

    // On estime la capacité uniquement sur R1.
    // En Paired-End, 1 paire = 1 fragment = 1 hachage, donc la taille de R1 suffit !
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
        println!("Capacité totale de la table de hachage estimée à {} fragments", capacite_totale);
    }

    let debut = Instant::now();

    // Logique d'aiguillage Single-End vs Paired-End
    let (traitees, duplications) = if let Some(entree_r2) = &arguments.entree_r2 {

        // On s'assure que l'utilisateur a bien fourni un fichier de sortie pour R2
        let sortie_r2 = arguments.sortie_r2.as_ref().expect(
            "Erreur critique : L'argument --sortie-r2 (-p) est obligatoire lorsque --entree-r2 (-2) est utilisé."
        );

        if arguments.verbeux {
            println!("Fichier d'entrée secondaire (R2) : {}", entree_r2);
            println!("Fichier de sortie secondaire (R2) : {}", sortie_r2);
            println!("--- Lancement du traitement Paired-End ---");
        }

        distribuer_paire(
            &arguments.entree,
            entree_r2,
            &arguments.sortie,
            sortie_r2,
            capacite_totale,
            arguments.forcer,
            arguments.verbeux,
            arguments.simulation,
            type_de_hachage_selectionne,
        )?
    } else {
        if arguments.verbeux {
            println!("--- Lancement du traitement Single-End ---");
        }

        distribuer(
            &arguments.entree,
            &arguments.sortie,
            capacite_totale,
            arguments.forcer,
            arguments.verbeux,
            arguments.simulation,
            type_de_hachage_selectionne,
        )?
    };

    if arguments.verbeux {
        println!(
            "Fragments traités : {}\nDoublons supprimés : {:.2}%",
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