use std::collections::BTreeMap;
use serde::Serialize;

pub fn show(json: bool) {
    let devices = match pci_devices() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return;
        }
    };

    let mut map: BTreeMap<i32, Vec<_>> = BTreeMap::new();

    for d in devices {
        map.entry(d.numa).or_default().push(d);
    }

    for devs in map.values_mut() {
        devs.sort_by(|a, b| a.bdf.cmp(&b.bdf));
    }

    if json {
        match serde_json::to_string_pretty(&map) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("failed to serialize json: {}", e),
        }
        return;
    }

    println!("NUMA TOPOLOGY\n");

    for (numa, devs) in map {
        println!("NUMA {}", numa);
        for d in devs {
            println!(" └─ {} {}", d.bdf, d.desc);
        }
    }
}

// 复用 pci 内部逻辑（简化处理）
fn pci_devices() -> Result<Vec<SimpleDev>, String> {
    if !cfg!(target_os = "linux") {
        return Err("pci-topo currently requires Linux (lspci + /sys)".to_string());
    }

    let mut res = Vec::new();

    let text = run_command("lspci", &[])?;

    for line in text.lines() {
        let bdf = line.split_whitespace().next().unwrap_or("").to_string();
        let path = sysfs_path(&bdf);

        let numa = std::fs::read_to_string(format!("{}/numa_node", path))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(-1);

        res.push(SimpleDev {
            bdf,
            desc: line.to_string(),
            numa,
        });
    }

    Ok(res)
}

#[derive(Serialize)]
struct SimpleDev {
    bdf: String,
    desc: String,
    numa: i32,
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
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
