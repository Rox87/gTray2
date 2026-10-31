//! klisten.rs - Formatação e captura de combinações de teclas.

pub fn format_key_combination<S: AsRef<str>>(keys: &[S]) -> String {
    if keys.is_empty() {
        return String::new();
    }

    let mut modifiers = Vec::new();
    let mut non_modifiers = Vec::new();

    for key_ref in keys {
        let key = key_ref.as_ref();
        let k_lower = key.to_lowercase();
        match k_lower.as_str() {
            "ctrl" | "control" => modifiers.push((0, "Ctrl")),
            "alt" => modifiers.push((1, "Alt")),
            "shift" => modifiers.push((2, "Shift")),
            "win" | "windows" => modifiers.push((3, "Win")),
            _ => {
                let formatted = if k_lower.starts_with('f') && k_lower[1..].chars().all(|c| c.is_ascii_digit()) {
                    k_lower.to_uppercase()
                } else if k_lower.len() == 1 {
                    k_lower.to_uppercase()
                } else {
                    let mut c = key.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                };
                non_modifiers.push(formatted);
            }
        }
    }

    modifiers.sort_by_key(|&(order, _)| order);
    modifiers.dedup_by_key(|(order, _)| *order);

    let mut parts: Vec<String> = modifiers.into_iter().map(|(_, name)| name.to_string()).collect();
    parts.extend(non_modifiers);

    parts.join("+")
}
