#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QListWidgetItem>
#include <QVector>

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

// Structure pour stocker les infos des navigateurs
struct BrowserInfo {
    QString name;
    QString pathSuffix;
    enum Type { CHROME, BRAVE, OPERA, VIVALDI, EDGE, FIREFOX, UNKNOWN } type;
};

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    void showScanOptions();
    void cleanFiles();

private:
    Ui::MainWindow *ui;

    // Liste des navigateurs supportés (déclarée une fois pour toute)
    QVector<BrowserInfo> browsers = {
        {"Chrome", "Google/Chrome", BrowserInfo::CHROME},
        {"Brave", "BraveSoftware/Brave-Browser", BrowserInfo::BRAVE},
        {"Opera", "Opera Software/Opera Stable", BrowserInfo::OPERA},
        {"Vivaldi", "Vivaldi", BrowserInfo::VIVALDI},
        {"Edge", "Microsoft/Edge", BrowserInfo::EDGE},
        {"Firefox", "Mozilla/Firefox", BrowserInfo::FIREFOX}
    };

    // Variables pour gérer l'état du scan
    bool isScanning = false;

    void scanFiles(bool scanTempFiles, bool scanBrowserCookies);
    void scanDirectory(const QString& dirPath);
    void scanBrowserCookies_internal();
    bool isUnnecessaryCookie(const QString& cookieFilePath);
    bool isBrowserRunning(const QString& browserPath, BrowserInfo::Type type);
};
#endif // MAINWINDOW_H
