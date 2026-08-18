use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserFamily {
    Chromium,
    Firefox,
}

#[derive(Debug, Clone, Copy)]
pub struct BrowserDef {
    pub display_name: &'static str,
    pub family: BrowserFamily,
    /// Chemin relatif a %LOCALAPPDATA% (Chromium) ou %APPDATA% (Firefox).
    pub vendor_path: &'static str,
    /// Noms de processus (comparaison insensible a la casse) surveilles
    /// pour savoir si le navigateur est actuellement ouvert.
    pub process_names: &'static [&'static str],
}

/// Navigateurs geres. Ajouter un navigateur = ajouter une ligne ici, sans
/// toucher a la logique de scan.
pub const fn browser_catalog() -> &'static [BrowserDef] {
    &[
        BrowserDef {
            display_name: "Chrome",
            family: BrowserFamily::Chromium,
            vendor_path: "Google/Chrome/User Data",
            process_names: &["chrome.exe"],
        },
        BrowserDef {
            display_name: "Brave",
            family: BrowserFamily::Chromium,
            vendor_path: "BraveSoftware/Brave-Browser/User Data",
            process_names: &["brave.exe"],
        },
        BrowserDef {
            display_name: "Edge",
            family: BrowserFamily::Chromium,
            vendor_path: "Microsoft/Edge/User Data",
            process_names: &["msedge.exe"],
        },
        BrowserDef {
            display_name: "Opera",
            family: BrowserFamily::Chromium,
            vendor_path: "Opera Software/Opera Stable",
            process_names: &["opera.exe"],
        },
        BrowserDef {
            display_name: "Vivaldi",
            family: BrowserFamily::Chromium,
            vendor_path: "Vivaldi/User Data",
            process_names: &["vivaldi.exe"],
        },
        BrowserDef {
            display_name: "Firefox",
            family: BrowserFamily::Firefox,
            vendor_path: "Mozilla/Firefox/Profiles",
            process_names: &["firefox.exe"],
        },
    ]
}

/// Sous-dossiers de profil d'un navigateur Chromium ("Default",
/// "Profile 1", "Profile 2", ...). Les autres entrees de `User Data`
/// (System Profile, Local State, Crashpad...) sont ignorees.
pub fn chromium_profiles(user_data_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(user_data_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name == "Default" || name.starts_with("Profile ")
        })
        .collect()
}

/// Un profil de dossier Firefox par sous-dossier de `Profiles`.
pub fn firefox_profiles(profiles_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(profiles_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect()
}
