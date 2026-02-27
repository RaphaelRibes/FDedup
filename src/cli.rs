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
    about = "Un outil de déduplication PCR FASTX rapide et économe en mémoire",
    before_help = ASCII_ART,
    arg_required_else_help = true
)]
#[command(help_expected = true)]
pub struct Cli {
    /// Chemin vers le fichier FASTX d'entrée
    #[arg(required = true)]
    pub entree: String,

    /// Chemin vers le fichier de sortie
    #[arg(default_value = "sortie.fastq.gz")]
    pub sortie: String,

    /// Forcer l'écrasement du fichier de sortie s'il existe
    #[arg(long, short)]
    pub forcer: bool,

    /// Activer les journaux verbeux
    #[arg(long, short)]
    pub verbeux: bool,

    /// Calculer le taux de duplication sans créer de fichier de sortie
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
