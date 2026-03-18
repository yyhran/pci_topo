#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum DeviceType {
    Nic,
    Gpu,
    Nvme,
    Other,
}

pub fn classify(desc: &str) -> DeviceType {
    let s = desc.to_lowercase();

    if s.contains("ethernet") || s.contains("network") || s.contains("mlx") {
        DeviceType::Nic
    } else if s.contains("nvidia") || s.contains("3d controller") || s.contains("vga") {
        DeviceType::Gpu
    } else if s.contains("nvme") {
        DeviceType::Nvme
    } else {
        DeviceType::Other
    }
}

impl DeviceType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nic" => Some(DeviceType::Nic),
            "gpu" => Some(DeviceType::Gpu),
            "nvme" => Some(DeviceType::Nvme),
            "other" => Some(DeviceType::Other),
            _ => None,
        }
    }
}
