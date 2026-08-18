use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use eframe::egui;
use hus_clean_core::{
    allowed_roots, clean_items, scan_browsers, scan_temp, temp_roots, Category, CleanOutcome,
    FoundItem, ScanOptions,
};

use crate::format::human_size;

const HUSKY_PNG: &[u8] = include_bytes!("../../../assets/husky.png");

struct Row {
    item: FoundItem,
    selected: bool,
    outcome: Option<CleanOutcome>,
}

enum WorkerMessage {
    Found(FoundItem),
    ScanDone,
    Cleaned(std::path::PathBuf, CleanOutcome),
    CleanDone,
}

pub struct HusCleanApp {
    rows: Vec<Row>,
    scanning: bool,
    cleaning: bool,
    status: String,
    show_scan_dialog: bool,
    show_confirm_dialog: bool,
    opt_temp: bool,
    opt_cookies: bool,
    opt_cache: bool,
    channel: Option<Receiver<WorkerMessage>>,
}

impl Default for HusCleanApp {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            scanning: false,
            cleaning: false,
            status: "Pret. Clique sur \"Scanner\" pour commencer.".to_string(),
            show_scan_dialog: false,
            show_confirm_dialog: false,
            opt_temp: true,
            opt_cookies: true,
            opt_cache: true,
            channel: None,
        }
    }
}

impl HusCleanApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }

    fn drain_channel(&mut self) {
        let Some(rx) = &self.channel else { return };
        let mut finished = false;
        for msg in rx.try_iter() {
            match msg {
                WorkerMessage::Found(item) => {
                    self.rows.push(Row { item, selected: true, outcome: None });
                }
                WorkerMessage::ScanDone => {
                    self.scanning = false;
                    finished = true;
                    self.status = if self.rows.is_empty() {
                        "Scan termine : aucun fichier trouve.".to_string()
                    } else {
                        format!("Scan termine : {} element(s) trouve(s).", self.rows.len())
                    };
                }
                WorkerMessage::Cleaned(path, outcome) => {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.item.path == path) {
                        row.outcome = Some(outcome);
                    }
                }
                WorkerMessage::CleanDone => {
                    self.cleaning = false;
                    finished = true;
                    let trashed = self
                        .rows
                        .iter()
                        .filter(|r| matches!(r.outcome, Some(CleanOutcome::Trashed)))
                        .count();
                    let skipped = self
                        .rows
                        .iter()
                        .filter(|r| matches!(r.outcome, Some(CleanOutcome::Skipped(_))))
                        .count();
                    let failed = self
                        .rows
                        .iter()
                        .filter(|r| matches!(r.outcome, Some(CleanOutcome::Failed(_))))
                        .count();
                    self.status = format!(
                        "Nettoyage termine : {trashed} envoye(s) a la corbeille, {skipped} ignore(s), {failed} erreur(s)."
                    );
                }
            }
        }
        if finished {
            self.channel = None;
        }
    }

    fn start_scan(&mut self, ctx: &egui::Context) {
        self.rows.clear();
        self.scanning = true;
        self.status = "Scan en cours...".to_string();

        let (tx, rx): (Sender<WorkerMessage>, Receiver<WorkerMessage>) = std::sync::mpsc::channel();
        self.channel = Some(rx);

        let opts = ScanOptions {
            temp_files: self.opt_temp,
            browser_cookies: self.opt_cookies,
            browser_cache: self.opt_cache,
        };
        let ctx = ctx.clone();

        thread::spawn(move || {
            if opts.temp_files {
                for root in temp_roots() {
                    for item in scan_temp(&root) {
                        if tx.send(WorkerMessage::Found(item)).is_err() {
                            return;
                        }
                        ctx.request_repaint();
                    }
                }
            }
            if opts.browser_cookies || opts.browser_cache {
                for item in scan_browsers(opts) {
                    if tx.send(WorkerMessage::Found(item)).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
            }
            let _ = tx.send(WorkerMessage::ScanDone);
            ctx.request_repaint();
        });
    }

    fn start_clean(&mut self, ctx: &egui::Context) {
        self.cleaning = true;
        self.status = "Nettoyage en cours...".to_string();

        let (tx, rx): (Sender<WorkerMessage>, Receiver<WorkerMessage>) = std::sync::mpsc::channel();
        self.channel = Some(rx);

        let selected: Vec<FoundItem> = self
            .rows
            .iter()
            .filter(|r| r.selected && r.outcome.is_none())
            .map(|r| r.item.clone())
            .collect();
        let roots = allowed_roots();
        let ctx = ctx.clone();

        thread::spawn(move || {
            for report in clean_items(&selected, &roots) {
                if tx
                    .send(WorkerMessage::Cleaned(report.path, report.outcome))
                    .is_err()
                {
                    return;
                }
                ctx.request_repaint();
            }
            let _ = tx.send(WorkerMessage::CleanDone);
            ctx.request_repaint();
        });
    }

    fn selected_count(&self) -> usize {
        self.rows.iter().filter(|r| r.selected && r.outcome.is_none()).count()
    }

    fn busy(&self) -> bool {
        self.scanning || self.cleaning
    }
}

fn category_tag(category: &Category) -> String {
    match category {
        Category::TempFile => "Temp".to_string(),
        Category::BrowserCookies { browser } => format!("Cookies - {browser}"),
        Category::BrowserCache { browser } => format!("Cache - {browser}"),
    }
}

fn outcome_text(outcome: &CleanOutcome) -> (String, egui::Color32) {
    match outcome {
        CleanOutcome::Trashed => ("Corbeille".to_string(), egui::Color32::from_rgb(120, 200, 120)),
        CleanOutcome::Skipped(reason) => (format!("Ignore : {reason}"), egui::Color32::from_rgb(230, 180, 90)),
        CleanOutcome::Failed(reason) => (format!("Erreur : {reason}"), egui::Color32::from_rgb(230, 100, 100)),
    }
}

impl eframe::App for HusCleanApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reapplique le theme sombre a chaque frame : eframe resynchronise
        // parfois les visuals sur le theme clair/sombre du systeme
        // d'exploitation, ce qui ecraserait sinon l'appel fait une seule
        // fois dans `new`.
        crate::theme::apply(ctx);
        self.drain_channel();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::from_bytes("bytes://husky.png", HUSKY_PNG)
                        .fit_to_exact_size(egui::vec2(160.0, 160.0)),
                );
                ui.vertical(|ui| {
                    ui.heading("Hus-Clean");
                    ui.label("Nettoyage de fichiers temporaires et de cookies indesirables");
                });
            });
            ui.add_space(6.0);
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_enabled_ui(!self.busy(), |ui| {
                    if ui.button("Scanner").clicked() {
                        self.show_scan_dialog = true;
                    }
                });
                ui.add_enabled_ui(!self.busy() && self.selected_count() > 0, |ui| {
                    if ui.button("Nettoyer").clicked() {
                        self.show_confirm_dialog = true;
                    }
                });
                if !self.rows.is_empty() {
                    if ui.button("Tout cocher").clicked() {
                        for row in &mut self.rows {
                            row.selected = true;
                        }
                    }
                    if ui.button("Tout decocher").clicked() {
                        for row in &mut self.rows {
                            row.selected = false;
                        }
                    }
                }
                if self.busy() {
                    ui.spinner();
                }
            });
            ui.label(&self.status);
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.rows.is_empty() && !self.scanning {
                ui.centered_and_justified(|ui| {
                    ui.label("Aucun element scanne pour le moment.");
                });
                return;
            }

            let busy = self.busy();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("items_grid")
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        for row in &mut self.rows {
                            let enabled = row.outcome.is_none() && !row.item.locked && !busy;
                            ui.add_enabled(enabled, egui::Checkbox::new(&mut row.selected, ""));
                            ui.label(category_tag(&row.item.category));
                            ui.label(&row.item.display_name);
                            ui.label(human_size(row.item.size_bytes));
                            if row.item.locked {
                                ui.colored_label(
                                    egui::Color32::from_rgb(230, 180, 90),
                                    "Verrouille (navigateur ouvert)",
                                );
                            } else if let Some(outcome) = &row.outcome {
                                let (text, color) = outcome_text(outcome);
                                ui.colored_label(color, text);
                            } else {
                                ui.label("");
                            }
                            ui.end_row();
                        }
                    });
            });
        });

        if self.show_scan_dialog {
            let mut open = true;
            egui::Window::new("Options de scan")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.opt_temp, "Fichiers temporaires");
                    ui.checkbox(&mut self.opt_cookies, "Cookies des navigateurs");
                    ui.checkbox(&mut self.opt_cache, "Cache des navigateurs");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let any = self.opt_temp || self.opt_cookies || self.opt_cache;
                        if ui.add_enabled(any, egui::Button::new("Lancer le scan")).clicked() {
                            self.show_scan_dialog = false;
                            self.start_scan(ctx);
                        }
                        if ui.button("Annuler").clicked() {
                            self.show_scan_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_scan_dialog = false;
            }
        }

        if self.show_confirm_dialog {
            let mut open = true;
            egui::Window::new("Confirmer le nettoyage")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Envoyer {} element(s) selectionne(s) a la corbeille ?",
                        self.selected_count()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Oui").clicked() {
                            self.show_confirm_dialog = false;
                            self.start_clean(ctx);
                        }
                        if ui.button("Non").clicked() {
                            self.show_confirm_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_confirm_dialog = false;
            }
        }
    }
}
