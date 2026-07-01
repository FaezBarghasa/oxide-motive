use clap::{Parser, Subcommand};
use std::process::Command;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build firmware for all targets
    BuildFirmware,
    /// Build the UI for the host
    BuildUi,
    /// Run the cloud backend
    RunCloud,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::BuildFirmware => {
            build_firmware()?;
        }
        Commands::BuildUi => {
            build_ui()?;
        }
        Commands::RunCloud => {
            run_cloud()?;
        }
    }

    Ok(())
}

fn build_firmware() -> Result<()> {
    let targets = ["thumbv7em-none-eabihf", "xtensa-esp32-none-elf"];
    for target in targets {
        let status = Command::new("cargo")
            .args(&["build", "--target", target, "-p", "oxide-firmware", "-p", "oxide-esp32-host"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to build for target {}", target);
        }
    }
    Ok(())
}

fn build_ui() -> Result<()> {
    let status = Command::new("cargo")
        .args(&["build", "-p", "oxide-hmi"])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to build the UI");
    }
    Ok(())
}

fn run_cloud() -> Result<()> {
    let status = Command::new("cargo")
        .args(&["run", "-p", "oxide-cloud"])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to run the cloud backend");
    }
    Ok(())
}