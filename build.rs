use clap::{Command, CommandFactory};
use clap_complete::{generate_to, shells};
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = "./target"; //idk if it works
    let bin_name = "resource-cache";

    let mut cmd: Command = Cli::command_for_update();
    let path = generate_to(
        shells::Bash,
        &mut cmd, // We need to specify what generator to use
        bin_name, // We need to specify the bin name manually
        outdir,  // We need to specify where to write to
    )?;

    println!("cargo:warning=completion file is generated: {path:?}");
    let path = generate_to(
        shells::Zsh,
        &mut cmd, // We need to specify what generator to use
        bin_name, // We need to specify the bin name manually
        outdir,  // We need to specify where to write to
    )?;

    println!("cargo:warning=completion file is generated: {path:?}");

    Ok(())
}