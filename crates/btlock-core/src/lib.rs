//! Core types and lock policy (v1 stub).

/// User-facing lock settings (defaults for v1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockSettings {
    pub delay_before_lock_secs: u32,
    pub max_attempts: u32,
    pub rssi_threshold_dbm: i16,
}

impl Default for LockSettings {
    fn default() -> Self {
        Self {
            delay_before_lock_secs: 5,
            max_attempts: 3,
            rssi_threshold_dbm: -70,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_sane() {
        let s = LockSettings::default();
        assert!(s.delay_before_lock_secs > 0);
        assert!(s.rssi_threshold_dbm < 0);
    }
}
