use crate::archive_reader::key_slot::KeySlot;
use crate::archive_reader::{Error, Result, SecretString};
use std::collections::BTreeSet;

/// Default maximum password-slot derivations in one open operation.
/// This is an operation limit, not a limit on valid persisted key directories.
pub const DEFAULT_PASSWORD_SLOT_BUDGET: u32 = 32;

/// Work admission shared by primary, mirror and explicit recovery attempts.
///
/// Construct a fresh budget for each password operation. A larger limit is an
/// explicit opt-in for trusted archives with many password slots; it changes
/// neither KDF parameters nor the archive format. Reusing the same budget
/// across attempts avoids repeating identical wrapping inputs that already
/// failed for this same credential. Successful unwraps are not suppressed.
pub struct PasswordWorkBudget<'a> {
    password: &'a SecretString,
    remaining: u32,
    seen: BTreeSet<(Vec<u8>, Vec<u8>)>,
}

impl<'a> PasswordWorkBudget<'a> {
    /// The default limit, bound to this credential for the operation lifetime.
    pub fn default_for(password: &'a SecretString) -> Self {
        Self::new(password, DEFAULT_PASSWORD_SLOT_BUDGET)
    }
    /// Admit at most `max_derivations` well-formed password-slot derivations.
    /// Zero deliberately admits no derivations.
    pub fn new(password: &'a SecretString, max_derivations: u32) -> Self {
        Self {
            password,
            remaining: max_derivations,
            seen: BTreeSet::new(),
        }
    }

    fn admit(&mut self, slot: &KeySlot) -> Result<bool> {
        let KeySlot::Password {
            salt,
            encrypted_key,
            ..
        } = slot
        else {
            return Ok(false);
        };
        if salt.len() != 16 || encrypted_key.len() < 16 {
            return Ok(false);
        }
        // These are public salt/ciphertext bytes, not password-derived secrets.
        let identity = (salt.clone(), encrypted_key.clone());
        if self.seen.contains(&identity) {
            return Ok(false);
        }
        if self.remaining == 0 {
            return Err(Error::SecurityLimitExceeded(
                "password slot work budget exhausted; explicitly raise the password-slot budget for a trusted archive".into(),
            ));
        }
        self.remaining -= 1;
        Ok(true)
    }

    pub(crate) fn try_slot(&mut self, slot: &KeySlot) -> Result<Option<Vec<u8>>> {
        if !self.admit(slot)? {
            return Ok(None);
        }
        match slot.try_password(self.password) {
            Ok(key) => Ok(Some(key)),
            Err(Error::InvalidKey) => {
                // Only a proven credential miss is reusable. A successful
                // unwrap must remain available to authenticate each candidate;
                // another successful derivation consumes another work unit.
                if let KeySlot::Password {
                    salt,
                    encrypted_key,
                    ..
                } = slot
                {
                    self.seen.insert((salt.clone(), encrypted_key.clone()));
                }
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slot(id: u64, marker: u8) -> KeySlot {
        KeySlot::Password {
            id,
            salt: vec![marker; 16],
            encrypted_key: vec![marker; 48],
        }
    }

    #[test]
    fn ledger_bounds_actual_derivations_without_discarding_successes() {
        let password = SecretString::try_from_bytes(b"ledger test".to_vec()).unwrap();
        let mut budget = PasswordWorkBudget::new(&password, 2);
        assert!(budget.admit(&slot(1, 1)).unwrap());
        assert!(budget.admit(&slot(999, 1)).unwrap());
        assert!(matches!(
            budget.admit(&slot(3, 3)),
            Err(Error::SecurityLimitExceeded(_))
        ));
        assert_eq!(budget.remaining, 0);
        assert_eq!(budget.seen.len(), 0);
        // Refusal does not mark an unattempted slot as successfully considered.
        assert!(matches!(
            budget.admit(&slot(3, 3)),
            Err(Error::SecurityLimitExceeded(_))
        ));
    }

    #[test]
    fn malformed_inputs_do_not_consume_work_and_override_is_explicit() {
        let malformed = KeySlot::Password {
            id: 1,
            salt: vec![0; 3],
            encrypted_key: vec![0; 48],
        };
        let password = SecretString::try_from_bytes(b"ledger test".to_vec()).unwrap();
        let mut zero = PasswordWorkBudget::new(&password, 0);
        assert!(!zero.admit(&malformed).unwrap());
        assert!(matches!(
            zero.admit(&slot(1, 1)),
            Err(Error::SecurityLimitExceeded(_))
        ));
        let mut explicit = PasswordWorkBudget::new(&password, 2);
        assert!(explicit.admit(&slot(1, 1)).unwrap());
        assert!(explicit.admit(&slot(2, 2)).unwrap());
    }

    #[test]
    fn only_proven_password_misses_are_shared_across_candidate_attempts() {
        let password = SecretString::try_from_bytes(b"correct ordinary password".to_vec()).unwrap();
        let wrong = SecretString::try_from_bytes(b"different ordinary password".to_vec()).unwrap();
        let slot = KeySlot::password_bytes(1, b"correct ordinary password", vec![7; 16], &[9; 32])
            .unwrap();
        let mut misses = PasswordWorkBudget::new(&wrong, 1);
        assert!(misses.try_slot(&slot).unwrap().is_none());
        assert!(misses.try_slot(&slot).unwrap().is_none());
        // A distinct credential has a distinct operation ledger, so the
        // previous failed candidate cannot suppress this successful unwrap.
        let mut successes = PasswordWorkBudget::new(&password, 2);
        for _ in 0..2 {
            let key = zeroize::Zeroizing::new(successes.try_slot(&slot).unwrap().unwrap());
            assert_eq!(key.as_slice(), &[9; 32]);
        }
        assert!(matches!(
            successes.try_slot(&slot),
            Err(Error::SecurityLimitExceeded(_))
        ));
    }
}
