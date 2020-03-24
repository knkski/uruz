mod error;

use rune::Rune;
use error::Error;
use std::fs::write;
use structopt::{self, clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
struct BuildConfig {
    #[structopt(help = "Path to bundle")]
    path: String,

    #[structopt(short = "o", long = "output", default_value = "rune.zip")]
    #[structopt(help = "Where to output the rune")]
    output_path: String,
}

/// Interact with a bundle and the runes contained therein.
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "AppSettings::TrailingVarArg"))]
#[structopt(raw(setting = "AppSettings::SubcommandRequiredElseHelp"))]
enum Config {
    #[structopt(name = "build")]
    Build(BuildConfig),
}

fn build(c: BuildConfig) -> Result<(), Error> {
    let rune = Rune::load(c.path)?;
    let zipped = rune.zip()?;
    write(c.output_path, zipped)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    match Config::from_args() {
        Config::Build(c) => build(c),
    }
}
