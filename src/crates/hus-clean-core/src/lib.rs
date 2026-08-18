//! Coeur metier de Hus-Clean : decouverte des fichiers temporaires et des
//! donnees de navigateurs a nettoyer, et suppression securisee (corbeille).
//!
//! Ce crate ne depend d'aucune bibliotheque graphique : toute la logique
//! est testable independamment de l'interface (crate `hus-clean-gui`), y
//! compris sur une machine Linux de developpement (les chemins Windows
//! sont lus depuis des variables d'environnement, jamais codes en dur).

mod browsers;
mod clean;
mod model;
mod scan;

pub use browsers::{browser_catalog, BrowserDef, BrowserFamily};
pub use clean::{clean_items, CleanOutcome, CleanReport};
pub use model::{Category, FoundItem};
pub use scan::{allowed_roots, scan_browsers, scan_temp, temp_roots, ScanOptions};
