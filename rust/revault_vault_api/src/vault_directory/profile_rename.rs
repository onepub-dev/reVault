use super::*;

impl VaultDirectory {
    /// Renames a local profile without changing its credentials or key history.
    /// Existing signed exchange transcripts retain their original display names.
    /// All credential moves, metadata and local access labels share one commit.
    pub fn rename_profile(&self, from: &str, to: &str) -> Result<()> {
        validate_vault_record_name(from)?;
        validate_vault_record_name(to)?;
        if !self.profile_exists(from)? {
            return Err(Error::NotFound(format!("vault profile {from}")));
        }
        if from == to {
            return Ok(());
        }
        if self.profile_exists(to)? {
            return Err(Error::AlreadyExists(format!("vault profile {to}")));
        }
        let ambiguous_contact = self.contact_exists(from)?;
        let mut lockbox = self.lockbox.borrow_mut();
        let prefixes = [
            (
                private_key_variable_name(from)?,
                private_key_variable_name(to)?,
            ),
            (
                owner_signing_key_variable_name(from)?,
                owner_signing_key_variable_name(to)?,
            ),
            (
                password_profiles::variable(from)?,
                password_profiles::variable(to)?,
            ),
        ];
        let mut moves = Vec::new();
        for (name, _) in lockbox.list_variables()? {
            for (old, new) in &prefixes {
                if name == *old {
                    moves.push((name.clone(), new.clone()));
                } else if let Some(suffix) = name
                    .as_str()
                    .strip_prefix(&format!("{}_GEN_", old.as_str()))
                {
                    moves.push((
                        name.clone(),
                        VariableName::new(format!("{}_GEN_{suffix}", new.as_str()))?,
                    ));
                }
            }
        }
        // Preflight every destination, including orphaned metadata, before staging.
        for (_, destination) in &moves {
            if lockbox.variable_sensitivity(destination)?.is_some() {
                return Err(Error::AlreadyExists(format!(
                    "profile record {destination}"
                )));
            }
        }
        let records = [
            (
                profile_history_record_path(from)?,
                profile_history_record_path(to)?,
            ),
            (
                profile_email_record_path(from)?,
                profile_email_record_path(to)?,
            ),
        ];
        for (_, destination) in &records {
            if lockbox.stat(destination).is_some() {
                return Err(Error::AlreadyExists(format!(
                    "profile record {destination}"
                )));
            }
        }
        let mut labels = Vec::new();
        if lockbox.stat(&LockboxPath::new("/access_slots")?).is_some() {
            for entry in lockbox.list(recursive_list("/access_slots")?)? {
                let entry = entry?;
                if entry.kind != LockboxEntryKind::File || !entry.path.ends_with(".lbas") {
                    continue;
                }
                let path = LockboxPath::new(entry.path)?;
                let mut label = decode_access_slot_label(&lockbox.get_file(&path)?)?;
                if label.name == from && ambiguous_contact {
                    return Err(Error::InvalidOperation(format!(
                        "access label {from} could identify a profile or contact; resolve the ambiguous access label before renaming"
                    )));
                }
                if label.name == from || label.name == format!("profile:{from}") {
                    label.name = format!("profile:{to}");
                    labels.push((path, encode_access_slot_label(&label)));
                }
            }
        }
        lockbox.move_variables(&moves)?;
        for (source, destination) in records {
            if lockbox.stat(&source).is_some() {
                lockbox.rename(&source, &destination)?;
            }
        }
        for (path, bytes) in labels {
            lockbox.add_file(&path, &bytes, true)?;
        }
        lockbox.commit()?;
        set_private_file_permissions(&self.path)?;
        drop(lockbox);
        let vault_id = self.path.to_string_lossy();
        let _ = crate::forget_owner_signing_key(&vault_id, from);
        let _ = crate::forget_owner_signing_key(&vault_id, to);
        Ok(())
    }
}
