use clap::{Parser, ValueEnum};

const ASCII_ART: &str = r#"
 /$$$$$$$$ /$$$$$$$                  /$$
| $$_____/| $$__  $$                | $$
| $$      | $$  \ $$  /$$$$$$   /$$$$$$$ /$$   /$$  /$$$$$$
| $$$$$   | $$  | $$ /$$__  $$ /$$__  $$| $$  | $$ /$$__  $$
| $$__/   | $$  | $$| $$$$$$$$| $$  | $$| $$  | $$| $$  \ $$
| $$      | $$  | $$| $$_____/| $$  | $$| $$  | $$| $$  | $$
| $$      | $$$$$$$/|  $$$$$$$|  $$$$$$$|  $$$$$$/| $$$$$$$/
|__/      |_______/  \_______/ \_______/ \______/ | $$____/
                                                  | $$
                                                  | $$
                                                  |__/
"#;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Un outil de déduplication PCR FASTX rapide et économe en mémoire (Mode Paired-End)",
    before_help = ASCII_ART,
    arg_required_else_help = true
)]
#[command(help_expected = true)]
pub struct Cli {
    /// Chemin vers le fichier FASTX d'entrée (R1 ou Single-End)
    #[arg(required = true, short = '1', long)]
    pub entree: String,

    /// Chemin vers le fichier FASTX d'entrée R2 (Optionnel, active le mode Paired-End)
    #[arg(short = '2', long)]
    pub entree_r2: Option<String>,

    /// Chemin vers le fichier de sortie (R1 ou Single-End)
    #[arg(short = 'o', long, default_value = "sortie_R1.fastq.gz")]
    pub sortie: String,

    /// Chemin vers le fichier de sortie R2 (Requis si --entree-r2 est fourni)
    #[arg(short = 'p', long)]
    pub sortie_r2: Option<String>,

    /// Forcer l'écrasement des fichiers de sortie si ils existent
    #[arg(long, short)]
    pub forcer: bool,

    /// Activer les journaux verbeux
    #[arg(long, short)]
    pub verbeux: bool,

    /// Calculer le taux de duplication sans créer de fichiers de sortie
    #[arg(long, short = 's')]
    pub simulation: bool,

    /// Seuil pour la sélection automatique de la taille de hachage (ignoré si --hachage est défini)
    #[arg(long, short = 'l', default_value_t = 0.01)]
    pub seuil: f64,

    /// Spécifier manuellement la taille de hachage (64 ou 128 bits)
    #[arg(long, short = 'H')]
    pub hachage: Option<ModeHachage>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ModeHachage {
    #[value(name = "64")]
    Bit64,
    #[value(name = "128")]
    Bit128,
}