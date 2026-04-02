use crate::drift::DriftLevel;

pub fn advisory_message(level: DriftLevel) -> &'static str {
    match level {
        DriftLevel::Benign =>
            "✅ Structure evolving safely.",
        DriftLevel::Strategic =>
            "🧭 Strategic expansion detected — confirm architectural direction.",
        DriftLevel::Harmful =>
            "⚠️ Structural integrity weakening: consider re-modularization.",
    }
}
