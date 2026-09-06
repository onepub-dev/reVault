use revault_browser_protocol::Error;
use std::{
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

struct Dialog(Child);
impl Drop for Dialog {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(super) fn approve(text: &str, cancelled: &AtomicBool, expires: u64) -> Result<(), Error> {
    // Fixed executable and literal argv: no shell, markup or website-rendered UI.
    let epoch = super::service::EPOCH.load(Ordering::SeqCst);
    if cancelled.load(Ordering::SeqCst) {
        return Err(Error::Cancelled);
    }
    let child = Command::new("/usr/bin/zenity")
        .args([
            "--question",
            "--default-cancel",
            "--no-markup",
            "--title=reVault authorisation",
            "--ok-label=Approve",
            "--cancel-label=Deny",
            "--text",
            text,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| Error::ApprovalRequired)?;
    let mut child = Dialog(child);
    loop {
        let state = if cancelled.load(Ordering::SeqCst)
            || epoch != super::service::EPOCH.load(Ordering::SeqCst)
        {
            Err(Error::Cancelled)
        } else if super::now()? >= expires {
            Err(Error::Expired)
        } else {
            Ok(())
        };
        if let Err(error) = state {
            let _ = child.0.kill();
            let _ = child.0.wait();
            return Err(error);
        }
        if let Some(status) = child.0.try_wait().map_err(|_| Error::ApprovalRequired)? {
            return if status.success() {
                Ok(())
            } else {
                Err(Error::Denied)
            };
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
