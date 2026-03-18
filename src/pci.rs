use std::fs;
use std::process::Command;

use crate::classify::{classify, DeviceType};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct PciDevice {
    bdf: String,
    numa: i32,
    cpulist: String,
    desc: String,
    dtype: DeviceType,
}

pub fn scan(filter: Option<String>, dev_type: Option<String>, json: bool) {
    let mut devices = match collect_devices() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return;
        }
    };

    // filter keyword
    if let Some(f) = filter {
        devices.retain(|d| d.desc.to_lowercase().contains(&f.to_lowercase()));
    }

    // filter type
    if let Some(t) = dev_type {
        if let Some(t) = DeviceType::from_str(&t) {
            devices.retain(|d| d.dtype == t);
        }
    }

    if json {
        match serde_json::to_string_pretty(&devices) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("failed to serialize json: {}", e),
        }
        return;
    }

    devices.sort_by(|a, b| a.bdf.cmp(&b.bdf));

    println!("{:<15} {:<6} {:<5} {:<12} {}", "BDF", "TYPE", "NUMA", "CPU", "DEVICE");

    for d in devices {
        println!(
            "{:<15} {:<6} {:<5} {:<12} {}",
            d.bdf,
            format!("{:?}", d.dtype),
            d.numa,
            d.cpulist,
            d.desc
        );
    }
}

fn collect_devices() -> Result<Vec<PciDevice>, String> {
    if !cfg!(target_os = "linux") {
        return Err("pci-topo currently requires Linux (lspci + /sys)".to_string());
    }

    let mut result = Vec::new();

    let text = run_command("lspci", &[])?;

    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let bdf = match parts.next() {
            Some(v) => v.to_string(),
            None => continue,
        };

        let desc = line.to_string();
        let dtype = classify(&desc);

        let sys_path = sysfs_path(&bdf);

        let numa = read_int(format!("{}/numa_node", sys_path));
        let cpulist = read_str(format!("{}/local_cpulist", sys_path));

        result.push(PciDevice {
            bdf,
            numa,
            cpulist,
            desc,
            dtype,
        });
    }

    Ok(result)
}

fn read_int(path: String) -> i32 {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(-1)
}

fn read_str(path: String) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| "-".into())
        .trim()
        .to_string()
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {}", cmd, e))?;

    if !output.status.success() {
        return Err(format!("{} exited with {}", cmd, output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn sysfs_path(bdf: &str) -> String {
    let colon_count = bdf.matches(':').count();
    if colon_count >= 2 {
        format!("/sys/bus/pci/devices/{}", bdf)
    } else {
        format!("/sys/bus/pci/devices/0000:{}", bdf)
    }
}
