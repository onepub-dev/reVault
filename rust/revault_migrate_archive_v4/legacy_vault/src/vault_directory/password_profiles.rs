use super::*;

const PREFIX: &str = "LOCKBOX_VAULT_PASSWORD_PROFILE_";

fn variable(name: &str) -> Result<VariableName> {
    validate_vault_record_name(name)?;
    VariableName::new(format!("{PREFIX}{}", crate::encode_hex(name.as_bytes())))
}

impl VaultDirectory {
    /// Whether either kind of profile exists under this name.
    pub fn profile_exists(&self, name: &str) -> Result<bool> {
        Ok(self.private_key_exists(name)? || self.password_profile_exists(name)?)
    }

    /// Whether a password profile exists under this name.
    pub fn password_profile_exists(&self, name: &str) -> Result<bool> {
        Ok(self
            .lockbox
            .borrow()
            .variable_sensitivity(&variable(name)?)?
            .is_some())
    }

    /// Lists all profile names without exposing credentials.
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut names = self.list_private_keys()?;
        names.extend(self.list_password_profiles()?);
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Lists password profile names without exposing credentials.
    pub fn list_password_profiles(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for (name, _) in self.lockbox.borrow().list_variables()? {
            if let Some(encoded) = name.as_str().trim_start_matches('/').strip_prefix(PREFIX) {
                let bytes = crate::decode_hex(encoded)
                    .map_err(|err| Error::CorruptVaultRecord(err.to_string()))?;
                let name = String::from_utf8(bytes).map_err(|_| {
                    Error::CorruptVaultRecord("invalid password profile name".into())
                })?;
                validate_vault_record_name(&name)?;
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }

    /// Creates a profile containing a generated 256-bit random password.
    pub fn create_password_profile(&self, name: &str) -> Result<()> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|err| Error::Io(err.to_string()))?;
        let mut encoded = crate::encode_hex(&bytes);
        use zeroize::Zeroize;
        bytes.zeroize();
        let password = SecretString::try_from_bytes(encoded.as_bytes().to_vec());
        encoded.zeroize();
        self.store_password_profile(name, &password?, false)
    }

    /// Stores a password profile. Replacement is restricted to the same type.
    pub fn store_password_profile(
        &self,
        name: &str,
        password: &SecretString,
        overwrite: bool,
    ) -> Result<()> {
        let variable = variable(name)?;
        if self.private_key_exists(name)? || (!overwrite && self.password_profile_exists(name)?) {
            return Err(Error::AlreadyExists(format!("vault profile {name}")));
        }
        if password.with_bytes(|bytes| bytes.is_empty())? {
            return Err(Error::InvalidInput(
                "profile password must not be empty".into(),
            ));
        }
        self.put_secret_variable_record(&variable, password)
    }

    /// Retrieves the password into protected memory.
    pub fn load_profile_password(&self, name: &str) -> Result<SecretString> {
        self.lockbox
            .borrow()
            .with_secret_variable(&variable(name)?, SecretString::try_clone)?
            .transpose()?
            .ok_or_else(|| Error::NotFound(format!("password profile {name}")))
    }

    /// Removes a profile credential locally. Existing lockbox access is unchanged.
    pub fn delete_profile(&self, name: &str) -> Result<()> {
        if self.password_profile_exists(name)? {
            self.delete_secret_variable_record_if_exists(&variable(name)?)
        } else {
            self.delete_private_key(name)
        }
    }
}
