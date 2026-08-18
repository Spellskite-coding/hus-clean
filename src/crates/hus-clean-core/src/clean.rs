use std::path::PathBuf;

use crate::model::FoundItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanOutcome {
    /// Envoye avec succes a la corbeille (jamais de suppression definitive).
    Trashed,
    Skipped(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct CleanReport {
    pub path: PathBuf,
    pub display_name: String,
    pub outcome: CleanOutcome,
}

/// Envoie les elements fournis a la corbeille Windows (jamais de
/// suppression definitive), en verifiant pour chacun que son chemin
/// canonique reste bien a l'interieur d'un des dossiers autorises
/// (`allowed_roots`).
///
/// C'est une deuxieme ligne de defense, independante du scan : meme si un
/// bug de scan laissait passer un chemin inattendu (lien symbolique,
/// jonction NTFS plantee dans un dossier temp par un autre processus...),
/// aucune suppression ne peut sortir des racines autorisees.
pub fn clean_items(items: &[FoundItem], allowed_roots: &[PathBuf]) -> Vec<CleanReport> {
    let canonical_roots: Vec<PathBuf> = allowed_roots
        .iter()
        .filter_map(|r| std::fs::canonicalize(r).ok())
        .collect();

    items
        .iter()
        .map(|item| CleanReport {
            path: item.path.clone(),
            display_name: item.display_name.clone(),
            outcome: clean_one(item, &canonical_roots),
        })
        .collect()
}

fn clean_one(item: &FoundItem, canonical_roots: &[PathBuf]) -> CleanOutcome {
    if item.locked {
        return CleanOutcome::Skipped("verrouille (navigateur ouvert)".into());
    }

    // Ne jamais suivre un lien symbolique / une jonction : verification sur
    // le metadata *non suivi* du chemin d'origine, avant tout appel a
    // canonicalize (qui suit les liens par definition).
    match std::fs::symlink_metadata(&item.path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return CleanOutcome::Skipped("lien symbolique ignore par securite".into());
        }
        Err(_) => return CleanOutcome::Skipped("introuvable".into()),
        _ => {}
    }

    let canonical = match std::fs::canonicalize(&item.path) {
        Ok(p) => p,
        Err(_) => return CleanOutcome::Skipped("introuvable".into()),
    };

    let within_allowed_root = canonical_roots.iter().any(|root| canonical.starts_with(root));
    if !within_allowed_root {
        return CleanOutcome::Skipped("hors des dossiers autorises".into());
    }

    match trash::delete(&item.path) {
        Ok(()) => CleanOutcome::Trashed,
        Err(e) => CleanOutcome::Failed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use std::fs;

    fn item(path: PathBuf) -> FoundItem {
        FoundItem {
            display_name: path.display().to_string(),
            path,
            category: Category::TempFile,
            locked: false,
            size_bytes: None,
        }
    }

    #[test]
    fn refuses_to_delete_outside_allowed_roots() {
        let allowed = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("dont_touch_me.txt");
        fs::write(&victim, b"precious").unwrap();

        let reports = clean_items(&[item(victim.clone())], &[allowed.path().to_path_buf()]);

        assert_eq!(reports.len(), 1);
        assert!(matches!(reports[0].outcome, CleanOutcome::Skipped(_)));
        assert!(victim.exists(), "le fichier hors racine autorisee n'aurait jamais du etre touche");
    }

    #[test]
    fn refuses_to_delete_locked_items() {
        let allowed = tempfile::tempdir().unwrap();
        let file = allowed.path().join("cookies.sqlite");
        fs::write(&file, b"data").unwrap();

        let mut it = item(file.clone());
        it.locked = true;

        let reports = clean_items(&[it], &[allowed.path().to_path_buf()]);
        assert!(matches!(reports[0].outcome, CleanOutcome::Skipped(_)));
        assert!(file.exists());
    }

    #[cfg(unix)]
    #[test]
    fn never_follows_a_symlink() {
        let allowed = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("secret.txt");
        fs::write(&victim, b"precious").unwrap();

        let link = allowed.path().join("trap");
        std::os::unix::fs::symlink(&victim, &link).unwrap();

        let reports = clean_items(&[item(link)], &[allowed.path().to_path_buf()]);
        assert!(matches!(reports[0].outcome, CleanOutcome::Skipped(_)));
        assert!(victim.exists());
    }
}
