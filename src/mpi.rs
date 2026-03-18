use std::collections::BTreeMap;
use serde::Serialize;

pub fn plan(json: bool) {
    let text = match run_command("lscpu", &[]) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{}", msg);
            return;
        }
    };

    let mut map: BTreeMap<String, String> = BTreeMap::new();

    for line in text.lines() {
        if line.contains("NUMA node") && line.contains("CPU(s)") {
            let parts: Vec<_> = line.split(':').collect();
            if parts.len() == 2 {
                map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
    }

    if json {
        let plan = build_plan(&map);
        match serde_json::to_string_pretty(&plan) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("failed to serialize json: {}", e),
        }
        return;
    }

    println!("MPI PLAN\n");

    match build_plan(&map) {
        Ok(plan) => {
            for p in plan {
                println!(
                    "rank {}-{} -> {} (cpus {})",
                    p.rank_start, p.rank_end, p.node, p.cpus
                );
            }
        }
        Err(msg) => eprintln!("{}", msg),
    }
}

fn count_cpus(s: &str) -> Result<usize, String> {
    let mut total = 0;

    for part in s.split(',') {
        if let Some((a, b)) = part.split_once('-') {
            let a: usize = a.parse().map_err(|_| format!("invalid cpu range: {}", part))?;
            let b: usize = b.parse().map_err(|_| format!("invalid cpu range: {}", part))?;
            total += b - a + 1;
        } else {
            if part.trim().is_empty() {
                return Err("empty cpu list entry".to_string());
            }
            let _single: usize = part
                .trim()
                .parse()
                .map_err(|_| format!("invalid cpu id: {}", part))?;
            total += 1;
        }
    }

    Ok(total)
}

fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    if !cfg!(target_os = "linux") {
        return Err("pci-topo currently requires Linux (lscpu)".to_string());
    }

    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {}", cmd, e))?;

    if !output.status.success() {
        return Err(format!("{} exited with {}", cmd, output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[derive(Serialize)]
struct PlanEntry {
    rank_start: usize,
    rank_end: usize,
    node: String,
    cpus: String,
}

fn build_plan(map: &BTreeMap<String, String>) -> Result<Vec<PlanEntry>, String> {
    let mut plan = Vec::new();
    let mut rank = 0usize;

    for (node, cpus) in map {
        let count = count_cpus(cpus)?;
        if count == 0 {
            return Err(format!("no cpus found for {}", node));
        }
        plan.push(PlanEntry {
            rank_start: rank,
            rank_end: rank + count - 1,
            node: node.clone(),
            cpus: cpus.clone(),
        });
        rank += count;
    }

    Ok(plan)
}
