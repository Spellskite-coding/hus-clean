QT += core gui
greaterThan(QT_MAJOR_VERSION, 4): QT += widgets
CONFIG += c++17

TARGET = hus-clean
TEMPLATE = app

FORMS += mainwindow.ui
RESOURCES += ressources.qrc

HEADERS += mainwindow.h
SOURCES += main.cpp mainwindow.cpp
RC_FILE = hus-clean.rc
