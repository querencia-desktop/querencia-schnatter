//! Zoomstufen der Seite (Ansicht → Vergrößern/Verkleinern/Normale Größe),
//! dieselben Stufen wie in den gängigen Browsern.

pub const STUFEN: [f64; 13] = [
    0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0,
];
pub const KLEINSTER: f64 = STUFEN[0];
pub const GROESSTER: f64 = STUFEN[STUFEN.len() - 1];

/// `zoom` innerhalb der Stufen; NaN wird 1.
pub fn begrenzen(zoom: f64) -> f64 {
    if zoom.is_nan() {
        return 1.0;
    }
    zoom.clamp(KLEINSTER, GROESSTER)
}

/// Die nächstgrößere Stufe über `zoom` (bleibt an der größten stehen).
pub fn groesser(zoom: f64) -> f64 {
    STUFEN
        .iter()
        .copied()
        .find(|s| *s > zoom + 1e-3)
        .unwrap_or(GROESSTER)
}

/// Die nächstkleinere Stufe unter `zoom` (bleibt an der kleinsten stehen).
pub fn kleiner(zoom: f64) -> f64 {
    STUFEN
        .iter()
        .rev()
        .copied()
        .find(|s| *s < zoom - 1e-3)
        .unwrap_or(KLEINSTER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stufen_auf_und_ab() {
        assert_eq!(groesser(1.0), 1.1);
        assert_eq!(kleiner(1.0), 0.9);
        assert_eq!(groesser(3.0), 3.0);
        assert_eq!(kleiner(0.5), 0.5);
        // zwischen zwei Stufen geht es zur nächsten
        assert_eq!(groesser(1.2), 1.25);
        assert_eq!(kleiner(1.2), 1.1);
        assert_eq!(begrenzen(40.0), 3.0);
        assert_eq!(begrenzen(0.01), 0.5);
        assert_eq!(begrenzen(f64::NAN), 1.0);
    }
}
