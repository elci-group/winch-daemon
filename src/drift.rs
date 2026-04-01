use crate::fingerprint::Fingerprint;

#[derive(Debug)]
pub enum DriftLevel {
    Benign,
    Harmful,
    Strategic,
}

pub fn detect_drift(old: &Fingerprint, new: &Fingerprint) -> DriftLevel {
    let lost_traits = old.traits.iter().filter(|t| !new.traits.contains(t)).count();
    let new_structs = new.structs.iter().filter(|s| !old.structs.contains(s)).count();

    if lost_traits > 0 && new_structs > 0 {
        DriftLevel::Harmful
    } else if new_structs > 0 {
        DriftLevel::Strategic
    } else {
        DriftLevel::Benign
    }
}
