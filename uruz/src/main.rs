mod error;

use error::Error;
use liburuz::Rune;
use std::fs::write;
use structopt::{self, clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
struct BuildConfig {
    #[structopt(help = "Path to bundle")]
    path: String,

    #[structopt(short = "o", long = "output")]
    #[structopt(help = "Where to output the rune")]
    output_path: Option<String>,
}

/// Interact with a bundle and the runes contained therein.
#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::TrailingVarArg)]
#[structopt(setting = AppSettings::SubcommandRequiredElseHelp)]
enum Config {
    #[structopt(name = "build")]
    Build(BuildConfig),
}

fn build(c: BuildConfig) -> Result<(), Error> {
    let rune = Rune::load(c.path)?;
    let zipped = rune.zip()?;
    write(
        c.output_path
            .unwrap_or_else(|| format!("{}.rune", rune.metadata.name)),
        zipped,
    )?;
    Ok(())
}

fn main() -> Result<(), Error> {
    match Config::from_args() {
        Config::Build(c) => build(c),
    }
}
