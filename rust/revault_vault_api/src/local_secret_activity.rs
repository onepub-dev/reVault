//! Process-local suspend protection; never targets a remotely supplied PID.
use crate::sleep_watcher::{SleepEvent, SleepWatcher};
use std::io;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Default)]
struct State {
    active: usize,
    suspended: bool,
    unavailable: bool,
}
impl State {
    fn event(&mut self, event: SleepEvent) -> bool {
        match event {
            SleepEvent::SuspendRequested => self.suspended = true,
            SleepEvent::Resumed => self.suspended = false,
            SleepEvent::Unavailable => self.unavailable = true,
        }
        self.active != 0 && (self.suspended || self.unavailable)
    }
    fn enter(&mut self) -> io::Result<()> {
        if self.suspended || self.unavailable {
            return Err(io::Error::other("suspend protection is not ready"));
        }
        self.active = self
            .active
            .checked_add(1)
            .ok_or_else(|| io::Error::other("too many active secret scopes"))?;
        Ok(())
    }
}

static WATCHER: OnceLock<Result<Arc<Mutex<State>>, String>> = OnceLock::new();

fn observe(state: &Mutex<State>, event: SleepEvent) {
    // The callback can only terminate its own process. No numeric PID lookup,
    // stale process registration, or remotely selected target is involved.
    let terminate = state
        .lock()
        .map(|mut state| state.event(event))
        .unwrap_or(true);
    if terminate {
        std::process::exit(1);
    }
}

fn start() -> Result<Arc<Mutex<State>>, String> {
    let state = Arc::new(Mutex::new(State::default()));
    let callback = Arc::clone(&state);
    #[cfg(unix)]
    SleepWatcher::start_handler(move |event| observe(&callback, event))
        .map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        let watcher = SleepWatcher::start().map_err(|e| e.to_string())?;
        std::thread::Builder::new()
            .name("lockbox-local-suspend".into())
            .spawn(move || {
                while let Ok(event) = watcher.recv() {
                    observe(&callback, event);
                }
                observe(&callback, SleepEvent::Unavailable);
            })
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = callback;
        return Err("suspend notifications are unavailable on this platform".into());
    }
    #[allow(unreachable_code)]
    Ok(state)
}

pub(crate) struct LocalSecretActivity(Arc<Mutex<State>>);
impl LocalSecretActivity {
    pub(crate) fn begin() -> io::Result<Self> {
        let state = WATCHER.get_or_init(start).as_ref().map_err(|message| io::Error::other(format!(
            "requested suspend protection is unavailable: {message}; for an explicitly unprotected headless session set LOCKBOX_AGENT_TERMINATE_ON_SUSPEND=false and LOCKBOX_AGENT_PREVENT_SLEEP=false before starting the Session Agent"
        )))?;
        state
            .lock()
            .map_err(|_| io::Error::other("suspend state poisoned"))?
            .enter()?;
        Ok(Self(Arc::clone(state)))
    }
}
impl Drop for LocalSecretActivity {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.lock() {
            state.active = state.active.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_scopes_and_readiness_follow_policy_without_process_actions() {
        let mut state = State::default();
        state.enter().unwrap();
        state.enter().unwrap();
        state.active -= 1;
        assert!(state.event(SleepEvent::SuspendRequested));
        assert!(state.enter().is_err());
        state.active -= 1;
        assert!(!state.event(SleepEvent::SuspendRequested));
        assert!(!state.event(SleepEvent::Resumed));
        state.enter().unwrap();
        assert!(state.event(SleepEvent::Unavailable));
        state.active -= 1;
        assert!(state.enter().is_err());
    }
    #[test]
    fn unavailable_idle_watcher_cannot_admit_new_secret_scope() {
        let mut state = State::default();
        assert!(!state.event(SleepEvent::Unavailable));
        assert!(state.enter().is_err());
        assert_eq!(state.active, 0);
    }
}
