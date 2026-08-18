/// Formate une taille en octets en une chaine lisible (Ko, Mo, Go).
pub fn human_size(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return String::new();
    };
    const UNITS: [&str; 4] = ["o", "Ko", "Mo", "Go"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
