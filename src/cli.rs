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
    about = "A fast and memory-efficient FASTX PCR deduplication tool",
    before_help = ASCII_ART,
    arg_required_else_help = true
)]

#[command(help_expected = true)]
pub struct Cli {
    /// Path to the input FASTX file
    #[arg(required = true)]
    pub input: String,

    /// Path to the output file
    #[arg(default_value = "output.fastq.gz")]
    pub output: String,

    /// Force overwriting the output file if it exists
    #[arg(long, short)]
    pub force: bool,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,

    /// Calculate duplication rate without creating an output file
    #[arg(long, short = 'd')]
    pub dryrun: bool,

    /// Threshold for automatic hash size selection (ignored if --hash is set)
    #[arg(long, short, default_value_t = 0.01)]
    pub threshold: f64,

    /// Manually specify hash size (64 or 128 bits)
    #[arg(long, short = 'H')]
    pub hash: Option<HashMode>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum HashMode {
    #[value(name = "64")]
    Bit64,
    #[value(name = "128")]
    Bit128,
}