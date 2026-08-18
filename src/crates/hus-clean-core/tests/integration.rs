//! Tests d'integration bout-en-bout du coeur metier : simule une
//! arborescence "Windows-like" (TEMP / LOCALAPPDATA / APPDATA) sur disque
//! via des variables d'environnement, et verifie l'ensemble du pipeline
//! scan -> nettoyage.
//!
//! Toutes les mutations de variables d'environnement sont regroupees dans
//! un seul test pour eviter les courses entre tests executes en parallele
//! dans le meme processus (l'environnement d'un processus est un etat
//! partage entre threads en Rust).

use std::fs;
use std::path::Path;

use hus_clean_core::{
    allowed_roots, clean_items, scan_browsers, scan_temp, temp_roots, Category, CleanOutcome,
    FoundItem, ScanOptions,
};

fn write_file(path: &Path, contents: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn full_scan_and_clean_pipeline() {
    // Isole le HOME pour que la corbeille (via le crate `trash`, qui suit
    // la spec XDG sur Linux) n'ecrive jamais dans le vrai
    // ~/.local/share/Trash de la machine qui execute les tests.
    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());

    let root = tempfile::tempdir().unwrap();
    let local_appdata = root.path().join("Local");
    let appdata = root.path().join("Roaming");
    let temp_dir = local_appdata.join("Temp");

    std::env::set_var("TEMP", &temp_dir);
    std::env::set_var("LOCALAPPDATA", &local_appdata);
    std::env::set_var("APPDATA", &appdata);

    // --- Fichiers temporaires, y compris en sous-dossier ---
    write_file(&temp_dir.join("top.tmp"), b"a");
    write_file(&temp_dir.join("nested/deep/leaf.tmp"), b"b");

    // --- Chrome : deux profils, cookies modernes et legacy, cache ---
    let chrome_root = local_appdata.join("Google/Chrome/User Data");
    write_file(&chrome_root.join("Default/Network/Cookies"), b"cookies-default");
    write_file(&chrome_root.join("Default/Cache/data_0"), b"cache-bytes");
    write_file(&chrome_root.join("Profile 1/Cookies"), b"cookies-profile1");
    // Ne doit jamais etre traite comme un profil ni touche.
    write_file(&chrome_root.join("Local State"), b"{}");
    // Fichiers explicitement hors-perimetre : ne doivent jamais apparaitre
    // dans les resultats de scan, quelles que soient les options.
    write_file(&chrome_root.join("Default/History"), b"precious-history");
    write_file(&chrome_root.join("Default/Login Data"), b"precious-passwords");

    // --- Firefox : cookies + cache (le cache vit sous LOCALAPPDATA, les
    // cookies sous APPDATA, avec le meme nom de profil) ---
    let ff_profile_name = "abc123.default-release";
    write_file(
        &appdata.join(format!("Mozilla/Firefox/Profiles/{ff_profile_name}/cookies.sqlite")),
        b"ff-cookies",
    );
    write_file(
        &local_appdata.join(format!("Mozilla/Firefox/Profiles/{ff_profile_name}/cache2/entry1")),
        b"ff-cache",
    );

    // ---- Scan temp ----
    let roots = temp_roots();
    assert_eq!(roots, vec![temp_dir.clone()]);
    let temp_items: Vec<_> = roots.iter().flat_map(|r| scan_temp(r)).collect();
    let temp_names: Vec<_> = temp_items
        .iter()
        .map(|i| i.path.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    assert!(temp_names.contains(&"top.tmp".to_string()));
    assert!(temp_names.contains(&"leaf.tmp".to_string()), "le scan doit etre recursif");
    assert_eq!(temp_items.len(), 2);

    // ---- Scan navigateurs ----
    let browser_items = scan_browsers(ScanOptions {
        temp_files: false,
        browser_cookies: true,
        browser_cache: true,
    });

    let cookie_items: Vec<_> = browser_items
        .iter()
        .filter(|i| matches!(i.category, Category::BrowserCookies { .. }))
        .collect();
    assert_eq!(cookie_items.len(), 3, "2 profils Chrome + 1 profil Firefox");

    let cache_items: Vec<_> = browser_items
        .iter()
        .filter(|i| matches!(i.category, Category::BrowserCache { .. }))
        .collect();
    assert_eq!(cache_items.len(), 2, "1 dossier cache Chrome + 1 dossier cache2 Firefox");

    // "Local State" n'est pas un profil : on verifie indirectement que
    // seuls Default et Profile 1 ont ete parcourus, via le nombre exact
    // d'elements cookies/cache attendu ci-dessus (2 profils Chrome + 1
    // Firefox), et qu'aucun chemin ne mentionne "Local State".
    for item in &browser_items {
        assert!(!item.path.to_string_lossy().contains("Local State"));
    }

    // Aucun element ne doit jamais porter sur l'historique ou les mots de
    // passe : seuls les cookies et le cache sont des categories connues.
    for item in &browser_items {
        let path_str = item.path.to_string_lossy();
        assert!(!path_str.contains("History"));
        assert!(!path_str.contains("Login Data"));
    }

    // ---- Nettoyage ----
    let mut all_items = temp_items;
    all_items.extend(browser_items);
    let roots = allowed_roots();

    let reports = clean_items(&all_items, &roots);
    for report in &reports {
        assert_eq!(
            report.outcome,
            CleanOutcome::Trashed,
            "echec inattendu pour {}: {:?}",
            report.display_name,
            report.outcome
        );
    }

    // Les fichiers/dossiers ont bien disparu de leur emplacement d'origine
    // (envoyes a la corbeille, pas juste "marques" comme supprimes).
    for item in &all_items {
        assert!(!item.path.exists(), "{} aurait du etre nettoye", item.path.display());
    }

    // Le fichier hors-perimetre n'a jamais ete liste, donc jamais touche.
    assert!(chrome_root.join("Default/History").exists());
    assert!(chrome_root.join("Default/Login Data").exists());
    assert!(chrome_root.join("Local State").exists());
}

#[test]
fn clean_items_never_deletes_a_path_outside_any_known_root() {
    // Ce test ne touche a aucune variable d'environnement : il exerce
    // directement la liste blanche de clean_items avec des racines
    // explicites, sans dependre de TEMP/LOCALAPPDATA/APPDATA.
    let allowed = tempfile::tempdir().unwrap();
    let attacker_controlled = tempfile::tempdir().unwrap();

    let victim = attacker_controlled.path().join("do_not_touch.db");
    write_file(&victim, b"important");

    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());

    let item = FoundItem {
        display_name: victim.display().to_string(),
        path: victim.clone(),
        category: Category::TempFile,
        locked: false,
        size_bytes: None,
    };

    let reports = clean_items(&[item], &[allowed.path().to_path_buf()]);
    assert!(matches!(reports[0].outcome, CleanOutcome::Skipped(_)));
    assert!(victim.exists());
}
