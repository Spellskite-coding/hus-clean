fn main() {
    // N'embarque l'icone / le manifeste que pour la cible Windows : sur les
    // autres cibles (ex : verification rapide sur l'hote Linux), ce n'est
    // ni possible ni necessaire.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=../../assets/resources.rc");
        println!("cargo:rerun-if-changed=../../assets/husky.ico");
        println!("cargo:rerun-if-changed=../../assets/hus-clean.manifest");
        let _ = embed_resource::compile("../../assets/resources.rc", embed_resource::NONE);
    }
}
