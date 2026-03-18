mod pci;
mod topology;
mod mpi;
mod classify;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pci-topo")]
#[command(version)]
#[command(about = "PCI / NUMA topology inspector (HPC-oriented)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan PCI devices
    Scan {
        filter: Option<String>,

        #[arg(long)]
        r#type: Option<String>,
    },

    /// Show NUMA topology
    Numa,

    /// Generate MPI plan
    Mpi,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan { filter, r#type }) => {
            pci::scan(filter, r#type, cli.json);
        }
        Some(Commands::Numa) => {
            topology::show(cli.json);
        }
        Some(Commands::Mpi) => {
            mpi::plan(cli.json);
        }
        None => {
            pci::scan(None, None, cli.json);
        }
    }
}
