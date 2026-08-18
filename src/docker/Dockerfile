FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH

# Toolchain de build : mingw-w64 pour la cible Windows (GNU ABI), plus les
# outils pour fabriquer l'icone .ico a partir du SVG source.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        build-essential \
        pkg-config \
        mingw-w64 \
        binutils-mingw-w64-x86-64 \
        imagemagick \
        librsvg2-bin \
        icoutils \
        git \
        pkg-config \
        libssl-dev \
        clang \
        lld \
        llvm \
    && rm -rf /var/lib/apt/lists/*

# rustup + toolchain stable + cible windows-gnu (compile depuis Linux vers un
# .exe Windows, aucune modification du systeme hote).
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --profile minimal --default-toolchain stable --no-modify-path \
    && rustup target add x86_64-pc-windows-gnu x86_64-pc-windows-msvc \
    && rustup component add clippy \
    # Outils d'analyse statique (SAST) : clippy pour les erreurs/mauvais
    # patterns, cargo-audit pour les vulnerabilites connues (base RustSec)
    # dans les dependances, cargo-geiger pour reperer tout usage de code
    # `unsafe`.
    #
    # cargo-xwin permet de cross-compiler vers la cible MSVC (le toolchain
    # "officiel" Windows) depuis Linux, en telechargeant le SDK/CRT
    # Microsoft. Les binaires MSVC ont une empreinte bien plus familiere
    # pour les antivirus que les binaires mingw-w64 cross-compiles, qui
    # declenchent plus facilement les heuristiques anti-malware generiques.
    && cargo install --locked cargo-audit cargo-geiger cargo-xwin \
    # Le conteneur est lance avec --user <uid host> pour que les fichiers
    # ecrits dans /workspace (bind mount) restent la propriete de
    # l'utilisateur hote : CARGO_HOME/RUSTUP_HOME doivent donc etre
    # inscriptibles par n'importe quel uid.
    && chmod -R a+rwX /opt/cargo /opt/rustup

WORKDIR /workspace
