use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct DistroInfo {
    pub name: String,
    pub id: String,
    pub pkg_manager: String,
    pub version_id: Option<String>,
}

impl Default for DistroInfo {
    fn default() -> Self {
        Self {
            name: "Linux".to_string(),
            id: "linux".to_string(),
            pkg_manager: "unknown".to_string(),
            version_id: None,
        }
    }
}

pub fn detect() -> DistroInfo {
    let mut info = DistroInfo::default();

    let Ok(contents) = fs::read_to_string("/etc/os-release") else {
        return info;
    };

    let mut map: HashMap<String, String> = HashMap::new();
    for line in contents.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }

    if let Some(name) = map.get("NAME") {
        info.name = name.clone();
    }
    if let Some(id) = map.get("ID") {
        info.id = id.to_lowercase();
    }
    if let Some(ver) = map.get("VERSION_ID") {
        info.version_id = Some(ver.clone());
    }

    info.pkg_manager = pkg_manager_for(&info.id);
    info
}

fn pkg_manager_for(id: &str) -> String {
    match id {
        "nixos" => "nixos-rebuild / nix",
        "arch" | "artix" | "manjaro" | "endeavouros" | "cachyos" => "pacman",
        "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" | "zorin" => "apt",
        "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => "dnf",
        "opensuse-tumbleweed" | "opensuse-leap" | "sled" | "sles" => "zypper",
        "void" => "xbps",
        "alpine" => "apk",
        "gentoo" => "emerge",
        "slackware" => "slackpkg",
        _ => "unknown",
    }
    .to_string()
}
