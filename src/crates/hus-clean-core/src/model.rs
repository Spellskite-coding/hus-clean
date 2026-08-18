use std::path::PathBuf;

/// Categorie d'un element trouve lors du scan, utilisee par l'IHM pour
/// l'affichage et le regroupement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    /// Fichier trouve dans un dossier temporaire Windows.
    TempFile,
    /// Cookies d'un navigateur (fichier de base de donnees precis, jamais
    /// un dossier entier : on ne veut toucher que les cookies, pas
    /// l'historique, les mots de passe ou les extensions).
    BrowserCookies { browser: String },
    /// Cache / donnees de site d'un navigateur. Regenere automatiquement
    /// par le navigateur, donc sans risque a supprimer.
    BrowserCache { browser: String },
}

/// Un element candidat au nettoyage.
#[derive(Debug, Clone)]
pub struct FoundItem {
    /// Chemin reel sur le disque. Contrairement a la version C++ d'origine,
    /// ce chemin n'est jamais reconstruit a partir du texte affiche a
    /// l'utilisateur : il est porte directement par la structure.
    pub path: PathBuf,
    /// Texte lisible affiche dans la liste de l'IHM.
    pub display_name: String,
    pub category: Category,
    /// true si le navigateur proprietaire semble actuellement lance
    /// (processus detecte) : l'element est affiche mais jamais supprime.
    pub locked: bool,
    /// Taille en octets, si connue (affichage informatif).
    pub size_bytes: Option<u64>,
}
