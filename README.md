# pci-topo

PCI / NUMA topology inspector for HPC systems.

This is a vibe code project.

## Features

- PCI device discovery
- NUMA affinity (CPU / memory)
- GPU / NIC / NVMe detection
- MPI binding plan generation
- Optional JSON output (`--json`)
- Stable, deterministic output order

## Requirements

- Linux system
- `lspci` and `/sys` for PCI/NUMA data
- `lscpu` for MPI plan generation

If these tools are missing, the CLI will print a clear error message and exit.

## Install

```bash
git clone https://github.com/yyhran/pci_topo.git
cargo build --release
```

## Usage

```bash
# scan all devices
pci-topo

# scan with filter keyword
pci-topo scan mlx

# scan with device type
pci-topo scan --type gpu

# show NUMA topology
pci-topo numa

# generate MPI plan
pci-topo mpi

# JSON output for any subcommand
pci-topo --json
pci-topo scan --json
pci-topo numa --json
pci-topo mpi --json
```
