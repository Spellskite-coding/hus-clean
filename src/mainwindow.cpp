#include "mainwindow.h"
#include "ui_mainwindow.h"
#include <QDir>
#include <QFile>
#include <QMessageBox>
#include <QFileInfo>
#include <QListWidgetItem>
#include <QApplication>
#include <QPalette>
#include <QFont>
#include <QDebug>
#include <QDialog>
#include <QVBoxLayout>
#include <QCheckBox>
#include <QDialogButtonBox>
#include <QLabel>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    // Configuration de l'interface (style, police, etc.)
    QPalette palette;
    palette.setColor(QPalette::Window, QColor(53, 53, 53));
    palette.setColor(QPalette::WindowText, Qt::white);
    palette.setColor(QPalette::Base, QColor(30, 30, 30));
    palette.setColor(QPalette::AlternateBase, QColor(53, 53, 53));
    palette.setColor(QPalette::ToolTipBase, Qt::white);
    palette.setColor(QPalette::ToolTipText, Qt::white);
    palette.setColor(QPalette::Text, Qt::white);
    palette.setColor(QPalette::Button, QColor(53, 53, 53));
    palette.setColor(QPalette::ButtonText, Qt::white);
    palette.setColor(QPalette::BrightText, Qt::red);
    palette.setColor(QPalette::Link, QColor(42, 130, 218));
    palette.setColor(QPalette::Highlight, QColor(42, 130, 218));
    qApp->setPalette(palette);

    QFont font("Segoe UI", 10);
    qApp->setFont(font);

    this->setWindowTitle("Hus-Clean - Nettoyage de fichiers");
    this->resize(1024, 768);
    this->setWindowIcon(QIcon(":/husky.ico"));
    ui->centralwidget->setAutoFillBackground(true);

    connect(ui->scanButton, &QPushButton::clicked, this, &MainWindow::showScanOptions);
    connect(ui->cleanButton, &QPushButton::clicked, this, &MainWindow::cleanFiles);

    // État initial : bouton nettoyer désactivé
    ui->cleanButton->setEnabled(false);
}

MainWindow::~MainWindow()
{
    delete ui;
}

// Vérifie si un navigateur est en cours d'exécution (fichier verrouillé)
bool MainWindow::isBrowserRunning(const QString& browserPath, BrowserInfo::Type type)
{
    QString lockFile;
    switch (type) {
    case BrowserInfo::FIREFOX:
        lockFile = browserPath + "/.parentlock";
        break;
    default: // Chromium-based browsers
        lockFile = browserPath + "/Lockfile";
        break;
    }
    return QFile::exists(lockFile);
}

// Vérifie si un cookie est indésirable (simplifié en une ligne)
bool MainWindow::isUnnecessaryCookie(const QString& cookieFilePath)
{
    return cookieFilePath.contains("Cookies") || cookieFilePath.endsWith(".ldb") || cookieFilePath.endsWith(".log") || cookieFilePath.endsWith("cookies.sqlite");
}

void MainWindow::showScanOptions()
{
    // Créer le dialogue de sélection
    QDialog dialog(this);
    dialog.setWindowTitle("Options de scan");
    dialog.setMinimumWidth(300);

    QVBoxLayout* layout = new QVBoxLayout(&dialog);

    // Label d'instruction
    QLabel* label = new QLabel("Sélectionnez les éléments à scanner :");
    layout->addWidget(label);

    // Checkboxes
    QCheckBox* tempFilesCheck = new QCheckBox("Fichiers temporaires");
    QCheckBox* browserCookiesCheck = new QCheckBox("Cookies des navigateurs");
    tempFilesCheck->setChecked(true);
    browserCookiesCheck->setChecked(true);
    layout->addWidget(tempFilesCheck);
    layout->addWidget(browserCookiesCheck);

    // Boutons OK/Annuler
    QDialogButtonBox* buttonBox = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
    connect(buttonBox, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
    connect(buttonBox, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
    layout->addWidget(buttonBox);

    // Afficher le dialogue
    if (dialog.exec() == QDialog::Accepted) {
        // Vérifier qu'au moins une option est cochée
        if (!tempFilesCheck->isChecked() && !browserCookiesCheck->isChecked()) {
            QMessageBox::warning(this, "Attention", "Veuillez cocher au moins une option à scanner.");
            return;
        }

        // Lancer le scan avec les options sélectionnées
        scanFiles(tempFilesCheck->isChecked(), browserCookiesCheck->isChecked());
    }
}

void MainWindow::scanFiles(bool scanTempFiles, bool scanBrowserCookies)
{
    isScanning = true;

    // Désactiver les boutons pendant le scan
    ui->scanButton->setEnabled(false);
    ui->cleanButton->setEnabled(false);

    ui->fileListWidget->clear();

    // Scanner les dossiers temporaires seulement si demandé
    if (scanTempFiles) {
        QString tempPath = qgetenv("TEMP");
        if (!tempPath.isEmpty()) {
            scanDirectory(tempPath);
        }

        QString localTempPath = qgetenv("LOCALAPPDATA") + "/Temp";
        if (!localTempPath.isEmpty()) {
            scanDirectory(localTempPath);
        }
    }

    // Scanner les cookies des navigateurs seulement si demandé
    if (scanBrowserCookies) {
        scanBrowserCookies_internal();
    }

    isScanning = false;

    // Réactiver le bouton scanner
    ui->scanButton->setEnabled(true);
    // Activer le bouton nettoyer si des fichiers ont été trouvés
    ui->cleanButton->setEnabled(ui->fileListWidget->count() > 0);

    // Message de fin
    QString message = "Scan terminé !";
    if (ui->fileListWidget->count() == 0) {
        message += "\nAucun fichier trouvé.";
    } else {
        message += QString("\n%1 fichier(s) trouvé(s).").arg(ui->fileListWidget->count());
    }
    QMessageBox::information(this, "Scan terminé !", message);
}

void MainWindow::scanDirectory(const QString& dirPath)
{
    QDir dir(dirPath);
    if (!dir.exists()) return;

    // Lister les fichiers du dossier
    QFileInfoList files = dir.entryInfoList(QDir::Files | QDir::NoDotAndDotDot);
    for (const QFileInfo& file : files) {
        ui->fileListWidget->addItem(file.absoluteFilePath());
        ui->fileListWidget->scrollToBottom();
        QApplication::processEvents(); // Mise à jour de l'interface
    }
}

void MainWindow::scanBrowserCookies_internal()
{
    QString localAppData = qgetenv("LOCALAPPDATA");
    QString appData = qgetenv("APPDATA");

    for (const BrowserInfo& browser : browsers) {
        QString browserPath;
        if (browser.type == BrowserInfo::FIREFOX) {
            browserPath = appData + "/" + browser.pathSuffix + "/Profiles";
        } else {
            browserPath = localAppData + "/" + browser.pathSuffix + "/User Data/Default";
        }

        QDir browserDir(browserPath);
        if (!browserDir.exists()) continue;

        // Vérifier si le navigateur est en cours d'exécution
        if (isBrowserRunning(browserPath, browser.type)) {
            ui->fileListWidget->addItem("[" + browser.name + "] Fichiers verrouillés (fermez " + browser.name + ")");
            ui->fileListWidget->scrollToBottom();
            QApplication::processEvents();
            continue;
        }

        // Scanner les fichiers selon le type de navigateur
        if (browser.type == BrowserInfo::FIREFOX) {
            QFileInfoList profiles = browserDir.entryInfoList(QDir::Dirs | QDir::NoDotAndDotDot);
            for (const QFileInfo& profile : profiles) {
                QString cookiesPath = profile.absoluteFilePath() + "/cookies.sqlite";
                if (QFile::exists(cookiesPath)) {
                    ui->fileListWidget->addItem("[" + browser.name + "] " + QFileInfo(cookiesPath).fileName());
                    ui->fileListWidget->scrollToBottom();
                    QApplication::processEvents();
                }
            }
        } else {
            QFileInfoList files = browserDir.entryInfoList({"Cookies", "Cookies-journal", "*.ldb"}, QDir::Files);
            for (const QFileInfo& file : files) {
                if (isUnnecessaryCookie(file.absoluteFilePath())) {
                    ui->fileListWidget->addItem("[" + browser.name + "] " + file.fileName());
                    ui->fileListWidget->scrollToBottom();
                    QApplication::processEvents();
                }
            }
        }
    }
}

void MainWindow::cleanFiles()
{
    if (isScanning) {
        QMessageBox::warning(this, "Erreur", "Le scan n'est pas terminé. Attendez la fin du scan avant de nettoyer.");
        return;
    }

    if (ui->fileListWidget->count() == 0) {
        QMessageBox::warning(this, "Attention", "Aucun fichier à nettoyer. Lancez d'abord un scan.");
        return;
    }

    int ret = QMessageBox::question(this, "Confirmer le nettoyage", "Voulez-vous vraiment nettoyer ces fichiers ?",
                                     QMessageBox::Yes | QMessageBox::No);
    if (ret == QMessageBox::No) {
        return;
    }

    // Désactiver les boutons pendant le nettoyage
    ui->scanButton->setEnabled(false);
    ui->cleanButton->setEnabled(false);

    int deletedCount = 0;
    int errorCount = 0;

    for (int i = 0; i < ui->fileListWidget->count(); ++i) {
        QListWidgetItem* item = ui->fileListWidget->item(i);
        QString itemText = item->text();

        // Ignorer les fichiers verrouillés ou déjà traités
        if (itemText.contains("verrouillés") || itemText.contains(" - Supprimé") || itemText.contains(" - Erreur")) {
            continue;
        }

        QString filePath;

        // Vérifier si c'est un fichier navigateur (format "[Browser] fichier") ou un fichier temp (chemin complet)
        if (itemText.startsWith("[")) {
            // C'est un fichier de navigateur
            QString fileName = itemText.mid(itemText.indexOf("] ") + 2);
            QString browserName = itemText.mid(1, itemText.indexOf("]") - 1);

            // Reconstruire le chemin complet
            for (const BrowserInfo& browser : browsers) {
                if (browser.name == browserName) {
                    if (browser.type == BrowserInfo::FIREFOX) {
                        QString appData = qgetenv("APPDATA");
                        QString profilesPath = appData + "/" + browser.pathSuffix + "/Profiles/";
                        QDir profilesDir(profilesPath);
                        if (profilesDir.exists()) {
                            QFileInfoList profiles = profilesDir.entryInfoList(QDir::Dirs | QDir::NoDotAndDotDot);
                            for (const QFileInfo& profile : profiles) {
                                QString cookiesPath = profile.absoluteFilePath() + "/" + fileName;
                                if (QFile::exists(cookiesPath)) {
                                    filePath = cookiesPath;
                                    break;
                                }
                            }
                        }
                    } else {
                        QString localAppData = qgetenv("LOCALAPPDATA");
                        filePath = localAppData + "/" + browser.pathSuffix + "/User Data/Default/" + fileName;
                    }
                    break;
                }
            }
        } else {
            // C'est un fichier temporaire (chemin complet)
            filePath = itemText;
        }

        // Supprimer le fichier
        if (!filePath.isEmpty()) {
            QFile file(filePath);
            if (file.exists()) {
                if (file.remove()) {
                    item->setText(itemText + " - Supprimé");
                    deletedCount++;
                } else {
                    item->setText(itemText + " - En cours d'utilisation");
                    errorCount++;
                }
            }
        }
        QApplication::processEvents();
    }

    // Réactiver le bouton scanner
    ui->scanButton->setEnabled(true);
    // Le bouton nettoyer reste désactivé (fichiers déjà nettoyés)

    QString message = QString("Nettoyage terminé !\n%1 fichier(s) supprimé(s)").arg(deletedCount);
    if (errorCount > 0) {
        message += QString("\n%1 fichier(s) n'ont pas pu être nettoyés (en cours d'utilisation par le système)").arg(errorCount);
    }
    QMessageBox::information(this, "Nettoyage terminé !", message);
}
