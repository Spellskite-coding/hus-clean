use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

use crate::browsers::{browser_catalog, chromium_profiles, firefox_profiles, BrowserFamily};
use crate::model::{Category, FoundItem};

/// Ce que l'utilisateur a choisi de scanner.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScanOptions {
    pub temp_files: bool,
    pub browser_cookies: bool,
    pub browser_cache: bool,
}

/// Racines temp Windows standard, dedupliquees, ne gardant que celles qui
/// existent reellement sur le disque.
pub fn temp_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(temp) = std::env::var("TEMP") {
        roots.push(PathBuf::from(temp));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        roots.push(PathBuf::from(local).join("Temp"));
    }
    roots.retain(|p| p.exists());
    roots.sort();
    roots.dedup();
    roots
}

/// Toutes les racines sous lesquelles une suppression est autorisee :
/// dossiers temp + dossiers de navigateurs geres. Sert de liste blanche
/// pour `hus_clean_core::clean::clean_items`.
pub fn allowed_roots() -> Vec<PathBuf> {
    let mut roots = temp_roots();
    roots.extend(browser_allowed_roots());
    roots
}

fn browser_allowed_roots() -> Vec<PathBuf> {
    let local_appdata = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from);
    let appdata = std::env::var("APPDATA").ok().map(PathBuf::from);
    let mut roots = Vec::new();
    for def in browser_catalog() {
        match def.family {
            BrowserFamily::Chromium => {
                if let Some(local) = &local_appdata {
                    roots.push(local.join(def.vendor_path));
                }
            }
            BrowserFamily::Firefox => {
                if let Some(app) = &appdata {
                    roots.push(app.join(def.vendor_path));
                }
                if let Some(local) = &local_appdata {
                    roots.push(local.join("Mozilla/Firefox/Profiles"));
                }
            }
        }
    }
    roots.retain(|p| p.exists());
    roots
}

/// Parcourt recursivement `root` et retourne tous les fichiers trouves.
///
/// Les liens symboliques et jonctions ne sont jamais suivis : un dossier
/// temporaire est ecrit par n'importe quel processus utilisateur (pas
/// besoin de droits admin sur Windows pour y creer une jonction), donc un
/// lien piege qui redirigerait vers un autre dossier ne doit jamais etre
/// traverse par un outil de nettoyage.
pub fn scan_temp(root: &Path) -> Vec<FoundItem> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn walk(dir: &Path, out: &mut Vec<FoundItem>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, out);
        } else if file_type.is_file() {
            let size_bytes = entry.metadata().ok().map(|m| m.len());
            out.push(FoundItem {
                display_name: path.display().to_string(),
                path,
                category: Category::TempFile,
                locked: false,
                size_bytes,
            });
        }
    }
}

/// Scanne tous les navigateurs connus et retourne les cookies / caches
/// trouves selon `options`. Ne touche jamais a l'historique, aux mots de
/// passe, aux favoris ou aux extensions : seuls les fichiers de cookies et
/// les dossiers de cache, explicitement nommes, sont consideres.
pub fn scan_browsers(options: ScanOptions) -> Vec<FoundItem> {
    if !options.browser_cookies && !options.browser_cache {
        return Vec::new();
    }

    let local_appdata = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from);
    let appdata = std::env::var("APPDATA").ok().map(PathBuf::from);
    let running = running_process_names();

    let mut out = Vec::new();

    for def in browser_catalog() {
        let is_running = def
            .process_names
            .iter()
            .any(|name| running.contains(&name.to_lowercase()));

        match def.family {
            BrowserFamily::Chromium => {
                let Some(local) = &local_appdata else {
                    continue;
                };
                let user_data_dir = local.join(def.vendor_path);
                for profile in chromium_profiles(&user_data_dir) {
                    if options.browser_cookies {
                        // Chrome >= 96 stocke les cookies dans Network/Cookies ;
                        // les versions plus anciennes les stockaient a la
                        // racine du profil. On verifie les deux.
                        for candidate in [
                            profile.join("Network").join("Cookies"),
                            profile.join("Cookies"),
                        ] {
                            if candidate.is_file() {
                                out.push(cookie_item(candidate, def.display_name, is_running));
                            }
                        }
                    }
                    if options.browser_cache {
                        for cache_dir in [profile.join("Cache"), profile.join("Code Cache")] {
                            push_cache_dir(&cache_dir, def.display_name, is_running, &mut out);
                        }
                    }
                }
            }
            BrowserFamily::Firefox => {
                let Some(app) = &appdata else {
                    continue;
                };
                let profiles_dir = app.join(def.vendor_path);
                for profile in firefox_profiles(&profiles_dir) {
                    if options.browser_cookies {
                        let cookies = profile.join("cookies.sqlite");
                        if cookies.is_file() {
                            out.push(cookie_item(cookies, def.display_name, is_running));
                        }
                    }
                    if options.browser_cache {
                        if let (Some(local), Some(profile_name)) =
                            (&local_appdata, profile.file_name().and_then(|n| n.to_str()))
                        {
                            let cache_dir = local
                                .join("Mozilla/Firefox/Profiles")
                                .join(profile_name)
                                .join("cache2");
                            push_cache_dir(&cache_dir, def.display_name, is_running, &mut out);
                        }
                    }
                }
            }
        }
    }

    out
}

fn cookie_item(path: PathBuf, browser: &str, locked: bool) -> FoundItem {
    let size_bytes = path.metadata().ok().map(|m| m.len());
    FoundItem {
        display_name: format!("[{browser}] cookies - {}", path.display()),
        path,
        category: Category::BrowserCookies {
            browser: browser.to_string(),
        },
        locked,
        size_bytes,
    }
}

fn push_cache_dir(dir: &Path, browser: &str, locked: bool, out: &mut Vec<FoundItem>) {
    if !dir.is_dir() {
        return;
    }
    out.push(FoundItem {
        size_bytes: dir_size(dir),
        display_name: format!("[{browser}] cache - {}", dir.display()),
        path: dir.to_path_buf(),
        category: Category::BrowserCache {
            browser: browser.to_string(),
        },
        locked,
    });
}

fn dir_size(dir: &Path) -> Option<u64> {
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(entry.path());
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    Some(total)
}

fn running_process_names() -> HashSet<String> {
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new()),
    );
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
        .processes()
        .values()
        .filter_map(|p| p.name().to_str())
        .map(|n| n.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_temp_is_recursive_and_skips_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(tmp.path().join("top.tmp"), b"x").unwrap();
        fs::write(nested.join("deep.tmp"), b"y").unwrap();

        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), b"z").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), tmp.path().join("escape")).unwrap();

        let found = scan_temp(tmp.path());
        let names: Vec<_> = found
            .iter()
            .map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(names.contains(&"top.tmp".to_string()));
        assert!(names.contains(&"deep.tmp".to_string()));
        assert!(!names.contains(&"secret.txt".to_string()));
    }

    #[test]
    fn temp_roots_reads_env_and_filters_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let local = tmp.path().join("Local");
        fs::create_dir_all(local.join("Temp")).unwrap();

        std::env::set_var("TEMP", tmp.path().join("does-not-exist"));
        std::env::set_var("LOCALAPPDATA", &local);

        let roots = temp_roots();
        assert_eq!(roots, vec![local.join("Temp")]);
    }
}
