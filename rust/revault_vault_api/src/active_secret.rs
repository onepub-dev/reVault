use crate::agent_config::AgentConfig;
use crate::agent_log::log_agent_event;
use crate::agent_protocol::SecretActivityKind;
use crate::sleep_watcher::SleepInhibitor;
use std::collections::BTreeMap;
use std::io;
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct ActiveSecretProcess {
    pub(crate) pid: u32,
    pub(crate) kind: SecretActivityKind,
    started_at: Instant,
}

pub(crate) struct ActiveSecretRegistry {
    config: AgentConfig,
    processes: BTreeMap<u64, ActiveSecretProcess>,
    sleep_inhibitor: Option<SleepInhibitor>,
}

impl ActiveSecretRegistry {
    pub(crate) fn new(config: AgentConfig) -> Self {
        Self {
            config,
            processes: BTreeMap::new(),
            sleep_inhibitor: None,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    pub(crate) fn terminate_on_suspend(&self) -> bool {
        self.config.terminate_on_suspend
    }

    pub(crate) fn register(&mut self, pid: u32, kind: SecretActivityKind) -> io::Result<u64> {
        validate_pid(pid)?;
        let token = next_token(&self.processes)?;
        self.ensure_inhibitor()?;

        self.processes.insert(
            token,
            ActiveSecretProcess {
                pid,
                kind,
                started_at: Instant::now(),
            },
        );
        log_agent_event(format!(
            "secret activity registered pid={pid} kind={}",
            kind.as_str()
        ));
        Ok(token)
    }

    fn ensure_inhibitor(&mut self) -> io::Result<()> {
        if self.sleep_inhibitor.is_none() && self.config.prevent_sleep {
            match SleepInhibitor::acquire_active("reVault secret operation in progress") {
                Ok(inhibitor) => {
                    self.sleep_inhibitor = Some(inhibitor);
                    log_agent_event("active secret suspend inhibitor acquired");
                }
                Err(err) => return Err(io::Error::other(format!(
                    "requested sleep inhibition unavailable: {err}; explicitly set LOCKBOX_AGENT_PREVENT_SLEEP=false for an unprotected headless session"
                ))),
            }
        }
        Ok(())
    }

    pub(crate) fn resumed(&mut self) -> io::Result<()> {
        if !self.processes.is_empty() {
            self.ensure_inhibitor()?;
        }
        Ok(())
    }

    pub(crate) fn unregister(&mut self, pid: u32, token: u64) {
        let Some(process) = self.processes.get(&token) else {
            log_agent_event(format!("secret activity unregister missed pid={pid}"));
            return;
        };
        if process.pid != pid {
            log_agent_event(format!(
                "secret activity unregister rejected pid={pid} token={token}"
            ));
            return;
        }
        if let Some(process) = self.processes.remove(&token) {
            log_agent_event(format!(
                "secret activity unregistered pid={} kind={} age_ms={}",
                process.pid,
                process.kind.as_str(),
                elapsed_ms(process.started_at)
            ));
        }
        self.release_inhibitor_if_idle();
    }

    pub(crate) fn suspend_requested(&mut self) {
        let count = self.processes.len();
        if count == 0 {
            self.sleep_inhibitor.take();
            return;
        }
        if !self.config.terminate_on_suspend {
            log_agent_event(format!(
                "suspend requested; active secret processes retained count={count}"
            ));
            self.sleep_inhibitor.take();
            return;
        }

        // Active clients own their suspend watcher and self-termination. The
        // agent never sends signals to reusable numeric process identifiers.
        self.processes.clear();
        self.sleep_inhibitor.take();
    }

    fn release_inhibitor_if_idle(&mut self) {
        if self.processes.is_empty() && self.sleep_inhibitor.take().is_some() {
            log_agent_event("active secret suspend inhibitor released");
        }
    }
}

fn next_token(processes: &BTreeMap<u64, ActiveSecretProcess>) -> io::Result<u64> {
    loop {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).map_err(|err| io::Error::other(err.to_string()))?;
        let token = u64::from_le_bytes(bytes);
        if token != 0 && !processes.contains_key(&token) {
            return Ok(token);
        }
    }
}

fn elapsed_ms(started_at: Instant) -> u128 {
    started_at.elapsed().as_millis()
}

fn validate_pid(pid: u32) -> io::Result<()> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid individual process ID",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn registration_validates_individual_pid_before_state_changes() {
        let config = AgentConfig {
            prevent_sleep: false,
            terminate_on_suspend: false,
        };
        let mut registry = ActiveSecretRegistry::new(config);
        for pid in [0, i32::MAX as u32 + 1, u32::MAX] {
            assert_eq!(
                registry
                    .register(pid, SecretActivityKind::Open)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
            assert!(registry.is_empty());
        }
        assert!(validate_pid(1).is_ok());
        assert!(validate_pid(i32::MAX as u32).is_ok());
    }

    #[test]
    fn registry_registers_and_unregisters_without_inhibitor() {
        let config = AgentConfig {
            prevent_sleep: false,
            terminate_on_suspend: false,
        };
        let mut registry = ActiveSecretRegistry::new(config);
        let token = registry.register(1234, SecretActivityKind::Open).unwrap();
        assert!(!registry.is_empty());
        registry.unregister(1234, token);
        assert!(registry.is_empty());
    }

    #[test]
    fn suspend_clears_registry_when_termination_disabled() {
        let config = AgentConfig {
            prevent_sleep: false,
            terminate_on_suspend: false,
        };
        let mut registry = ActiveSecretRegistry::new(config);
        registry
            .register(1234, SecretActivityKind::Variables)
            .unwrap();
        registry.suspend_requested();
        assert!(!registry.is_empty());
    }

    #[test]
    fn elapsed_ms_is_monotonic_enough_for_logs() {
        let started = Instant::now() - Duration::from_millis(5);
        assert!(elapsed_ms(started) >= 5);
    }
}
