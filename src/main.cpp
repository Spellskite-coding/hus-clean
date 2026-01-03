#include <QApplication>
#include "mainwindow.h"
#include <QStyleFactory>

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);
    QApplication::setStyle("fusion");  // Force le style Fusion pour toute l'application
    MainWindow w;
    w.show();
    return a.exec();
}
