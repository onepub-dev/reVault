use std::io;
use std::thread;

#[cfg(windows)]
use std::sync::mpsc::Sender;
#[cfg(any(windows, test))]
use std::sync::mpsc::{self, Receiver};

#[cfg(unix)]
type SleepHandler = Box<dyn FnMut(SleepEvent) + Send>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SleepEvent {
    SuspendRequested,
    Resumed,
}

pub(crate) struct SleepWatcher {
    #[cfg(any(windows, test))]
    receiver: Receiver<SleepEvent>,
}

pub(crate) struct SleepInhibitor {
    _inner: platform::SleepInhibitor,
}

/// Platform sleep and suspend capabilities used by the Session Agent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentSleepSupport {
    /// True when the agent can receive suspend/resume notifications.
    pub suspend_notifications: bool,
    /// True when active secret operations can request temporary sleep inhibition.
    pub sleep_inhibition: bool,
}

impl AgentSleepSupport {
    /// True when all sleep/suspend management features are available.
    pub fn supported(self) -> bool {
        self.suspend_notifications && self.sleep_inhibition
    }
}

/// Returns sleep/suspend capabilities compiled for the current platform.
pub fn agent_sleep_support() -> AgentSleepSupport {
    platform::agent_sleep_support()
}

impl SleepInhibitor {
    pub(crate) fn acquire_active(reason: &str) -> io::Result<Self> {
        platform::SleepInhibitor::acquire_active(reason).map(|inner| Self { _inner: inner })
    }
}

impl SleepWatcher {
    #[cfg(windows)]
    pub(crate) fn start() -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        platform::spawn(sender)?;
        Ok(Self { receiver })
    }

    #[cfg(unix)]
    pub(crate) fn start_handler(
        handler: impl FnMut(SleepEvent) + Send + 'static,
    ) -> io::Result<()> {
        platform::spawn_handler(Box::new(handler))
    }

    #[cfg(all(test, unix))]
    pub(crate) fn drain(&self) -> Vec<SleepEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }

    #[cfg(windows)]
    pub(crate) fn recv(&self) -> Result<SleepEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    #[cfg(all(test, unix))]
    pub(crate) fn from_events(events: impl IntoIterator<Item = SleepEvent>) -> Self {
        let (sender, receiver) = mpsc::channel();
        for event in events {
            let _ = sender.send(event);
        }
        Self { receiver }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::time::Duration;
    #[cfg(test)]
    use zbus::blocking::connection::Builder;
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedFd;

    pub(super) fn spawn_handler(handler: SleepHandler) -> io::Result<()> {
        let connection = system_connection().map_err(io_error)?;
        spawn_handler_on(connection, handler)
    }

    fn spawn_handler_on(connection: Connection, handler: SleepHandler) -> io::Result<()> {
        let mut inhibitor = acquire_sleep_inhibitor(&connection).ok();
        // Register before returning success. The proxy filters by the service's
        // owner, object path and interface, rather than accepting arbitrary
        // signals from other bus participants.
        let proxy = logind_proxy(&connection).map_err(io_error)?;
        let signals = proxy.receive_signal("PrepareForSleep").map_err(io_error)?;
        drop(proxy);
        thread::Builder::new()
            .name("lockbox-sleep-watcher".to_string())
            .spawn(move || {
                let mut handler = handler;
                for message in signals {
                    let Ok((sleeping,)) = message.body().deserialize::<(bool,)>() else {
                        continue;
                    };
                    handler(if sleeping {
                        SleepEvent::SuspendRequested
                    } else {
                        SleepEvent::Resumed
                    });
                    // Keep the delay inhibitor until secret clearing completes.
                    if sleeping {
                        drop(inhibitor.take());
                    } else {
                        inhibitor = acquire_sleep_inhibitor(&connection).ok();
                    }
                }
            })
            .map(|_| ())
    }

    pub(super) fn agent_sleep_support() -> AgentSleepSupport {
        AgentSleepSupport {
            suspend_notifications: true,
            sleep_inhibition: true,
        }
    }

    pub(super) struct SleepInhibitor {
        _fd: OwnedFd,
    }

    impl SleepInhibitor {
        pub(super) fn acquire_active(reason: &str) -> io::Result<Self> {
            let connection = system_connection().map_err(io_error)?;
            acquire_logind_inhibitor(&connection, reason, "block")
                .map(|fd| Self { _fd: fd })
                .map_err(io_error)
        }
    }

    fn io_error(error: zbus::Error) -> io::Error {
        io::Error::other(error.to_string())
    }

    fn system_connection() -> zbus::Result<Connection> {
        bounded_connection(zbus::connection::Builder::system()?)
    }

    fn bounded_connection(builder: zbus::connection::Builder<'_>) -> zbus::Result<Connection> {
        // A method timeout starts after authentication. Bound connection setup
        // too, so an unresponsive bus cannot stall agent startup or secret use.
        let timeout = Duration::from_secs(5);
        async_io::block_on(futures_lite::future::or(
            async {
                builder
                    .method_timeout(timeout)
                    .build()
                    .await
                    .map(Connection::from)
            },
            async {
                async_io::Timer::after(timeout).await;
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "system D-Bus connection timed out after 5 seconds",
                )
                .into())
            },
        ))
    }

    fn logind_proxy(connection: &Connection) -> zbus::Result<Proxy<'_>> {
        Proxy::new(
            connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
    }

    fn acquire_sleep_inhibitor(connection: &Connection) -> zbus::Result<OwnedFd> {
        acquire_logind_inhibitor(
            connection,
            "Clear cached lockbox keys before system sleep",
            "delay",
        )
    }

    fn acquire_logind_inhibitor(
        connection: &Connection,
        reason: &str,
        mode: &str,
    ) -> zbus::Result<OwnedFd> {
        logind_proxy(connection)?.call("Inhibit", &("sleep", "lockbox", reason, mode))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::Read;
        use std::os::unix::net::UnixStream;
        use std::process::{Child, Command, Stdio};
        use std::sync::{mpsc, Mutex};
        use std::time::Instant;

        struct PrivateBus {
            child: Child,
            directory: std::path::PathBuf,
        }
        impl Drop for PrivateBus {
            fn drop(&mut self) {
                let _ = self.child.kill();
                let _ = self.child.wait();
                let _ = std::fs::remove_dir_all(&self.directory);
            }
        }

        struct Logind {
            inhibitors: Mutex<mpsc::Sender<(String, UnixStream)>>,
        }
        #[zbus::interface(name = "org.freedesktop.login1.Manager")]
        impl Logind {
            fn inhibit(&self, what: &str, who: &str, reason: &str, mode: &str) -> OwnedFd {
                assert_eq!(what, "sleep");
                assert_eq!(who, "lockbox");
                assert!(!reason.is_empty());
                let (read, write) = UnixStream::pair().unwrap();
                self.inhibitors
                    .lock()
                    .unwrap()
                    .send((mode.to_owned(), read))
                    .unwrap();
                OwnedFd::from(std::os::fd::OwnedFd::from(write))
            }
        }

        #[test]
        fn unresponsive_system_bus_handshake_is_bounded() {
            let directory = std::env::temp_dir().join(format!(
                "lbx-logind-stalled-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&directory).unwrap();
            let address = format!("unix:path={}", directory.join("bus").display());
            let listener = std::os::unix::net::UnixListener::bind(directory.join("bus")).unwrap();
            let started = Instant::now();
            let result =
                bounded_connection(zbus::connection::Builder::address(address.as_str()).unwrap());
            drop(listener);
            std::fs::remove_dir_all(directory).unwrap();
            assert!(result.is_err());
            assert!(started.elapsed() >= Duration::from_secs(4));
            assert!(started.elapsed() < Duration::from_secs(8));
        }

        #[test]
        fn private_logind_delivers_events_and_releases_inhibitor_after_handler() {
            // A private synthetic service is necessary: putting the real host
            // to sleep is neither a reliable nor an acceptable test operation.
            let directory = std::env::temp_dir().join(format!(
                "lbx-logind-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&directory).unwrap();
            let address = format!("unix:path={}", directory.join("bus").display());
            let child = Command::new("dbus-daemon")
                .args(["--session", "--nofork", &format!("--address={address}")])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("install dbus-daemon to run the private bus integration test");
            let _bus = PrivateBus { child, directory };
            let started = Instant::now();
            let client = loop {
                match Builder::address(address.as_str())
                    .unwrap()
                    .method_timeout(Duration::from_secs(2))
                    .build()
                {
                    Ok(connection) => break connection,
                    Err(error) => {
                        assert!(started.elapsed() < Duration::from_secs(5), "{error}");
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            };
            let (inhibitors, acquired) = mpsc::channel();
            let service = Builder::address(address.as_str())
                .unwrap()
                .name("org.freedesktop.login1")
                .unwrap()
                .serve_at(
                    "/org/freedesktop/login1",
                    Logind {
                        inhibitors: Mutex::new(inhibitors),
                    },
                )
                .unwrap()
                .build()
                .unwrap();
            let (events, received) = mpsc::channel();
            let (release, resume_handler) = mpsc::channel();
            spawn_handler_on(
                client.clone(),
                Box::new(move |event| {
                    events.send(event).unwrap();
                    if event == SleepEvent::SuspendRequested {
                        resume_handler.recv_timeout(Duration::from_secs(5)).unwrap();
                    }
                }),
            )
            .unwrap();
            let (mode, mut held) = acquired.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(mode, "delay");
            held.set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
            service
                .emit_signal(
                    None::<&str>,
                    "/org/freedesktop/login1",
                    "org.freedesktop.login1.Manager",
                    "PrepareForSleep",
                    &(true,),
                )
                .unwrap();
            assert_eq!(
                received.recv_timeout(Duration::from_secs(5)).unwrap(),
                SleepEvent::SuspendRequested
            );
            assert!(
                held.read(&mut [0]).is_err(),
                "inhibitor must remain held during clearing"
            );
            release.send(()).unwrap();
            held.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            assert_eq!(
                held.read(&mut [0]).unwrap(),
                0,
                "clearing releases inhibitor"
            );
            service
                .emit_signal(
                    None::<&str>,
                    "/org/freedesktop/login1",
                    "org.freedesktop.login1.Manager",
                    "PrepareForSleep",
                    &(false,),
                )
                .unwrap();
            assert_eq!(
                received.recv_timeout(Duration::from_secs(5)).unwrap(),
                SleepEvent::Resumed
            );
            let (mode, _held) = acquired.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(mode, "delay");
            let active = acquire_logind_inhibitor(&client, "test activity", "block").unwrap();
            let (mode, mut active_peer) = acquired.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(mode, "block");
            drop(active);
            active_peer
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            assert_eq!(active_peer.read(&mut [0]).unwrap(), 0);
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop};
    use objc2_io_kit::{
        io_connect_t, io_object_t, io_service_t, kIOMessageCanSystemSleep,
        kIOMessageSystemHasPoweredOn, kIOMessageSystemWillSleep, IOAllowPowerChange,
        IONotificationPort, IONotificationPortRef, IORegisterForSystemPower,
    };
    use std::ffi::{c_char, c_void, CString};
    use std::ptr::null_mut;
    use std::sync::Mutex;

    struct CallbackContext {
        handler: Mutex<SleepHandler>,
        root_port: io_connect_t,
    }

    pub(super) fn spawn_handler(handler: SleepHandler) -> io::Result<()> {
        thread::Builder::new()
            .name("lockbox-sleep-watcher".to_string())
            .spawn(move || watch_iokit(handler))
            .map(|_| ())
    }

    pub(super) fn agent_sleep_support() -> AgentSleepSupport {
        AgentSleepSupport {
            suspend_notifications: true,
            sleep_inhibition: true,
        }
    }

    pub(super) struct SleepInhibitor {
        assertion_id: u32,
    }

    impl SleepInhibitor {
        pub(super) fn acquire_active(reason: &str) -> io::Result<Self> {
            let assertion_type = CfString::new("NoIdleSleepAssertion")?;
            let reason = CfString::new(reason)?;
            let mut assertion_id = 0u32;
            // SAFETY: The CFString references are valid for the duration of
            // this call and `assertion_id` is a writable out pointer.
            let result = unsafe {
                IOPMAssertionCreateWithName(
                    assertion_type.as_raw(),
                    K_IOPM_ASSERTION_LEVEL_ON,
                    reason.as_raw(),
                    &mut assertion_id,
                )
            };
            if result == 0 {
                Ok(Self { assertion_id })
            } else {
                Err(io::Error::from_raw_os_error(result))
            }
        }
    }

    impl Drop for SleepInhibitor {
        fn drop(&mut self) {
            // SAFETY: `assertion_id` was returned by
            // `IOPMAssertionCreateWithName` and is released exactly once.
            unsafe {
                IOPMAssertionRelease(self.assertion_id);
            }
        }
    }

    struct CfString {
        raw: *const c_void,
    }

    impl CfString {
        fn new(value: &str) -> io::Result<Self> {
            let value = CString::new(value).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "string contains NUL byte")
            })?;
            // SAFETY: `value` is a valid NUL-terminated C string and the
            // returned object is owned by this wrapper.
            let raw = unsafe {
                CFStringCreateWithCString(
                    std::ptr::null(),
                    value.as_ptr(),
                    K_CF_STRING_ENCODING_UTF8,
                )
            };
            if raw.is_null() {
                Err(io::Error::other("failed to allocate CFString"))
            } else {
                Ok(Self { raw })
            }
        }

        fn as_raw(&self) -> *const c_void {
            self.raw
        }
    }

    impl Drop for CfString {
        fn drop(&mut self) {
            // SAFETY: `raw` is a Core Foundation object owned by this wrapper.
            unsafe {
                CFRelease(self.raw);
            }
        }
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
    }

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: *const c_void,
            level: u32,
            reason: *const c_void,
            assertion_id: *mut u32,
        ) -> i32;
        fn IOPMAssertionRelease(assertion_id: u32) -> i32;
    }

    fn watch_iokit(handler: SleepHandler) {
        let mut notification_port: IONotificationPortRef = null_mut();
        let mut notifier: io_object_t = 0;
        let context = Box::new(CallbackContext {
            handler: Mutex::new(handler),
            root_port: 0,
        });
        let context_ptr = Box::into_raw(context);
        // SAFETY: `context_ptr`, `notification_port`, and `notifier` are valid
        // for this registration call. The watcher thread runs the Core
        // Foundation loop for the process lifetime of the agent.
        let root_port = unsafe {
            IORegisterForSystemPower(
                context_ptr.cast::<c_void>(),
                &mut notification_port,
                Some(sleep_callback),
                &mut notifier,
            )
        };
        if root_port == 0 || notification_port.is_null() {
            // SAFETY: Reclaims the boxed context when registration failed.
            unsafe {
                drop(Box::from_raw(context_ptr));
            }
            return;
        }
        // SAFETY: The context allocation remains owned by this thread for as
        // long as the run loop is active.
        unsafe {
            (*context_ptr).root_port = root_port;
        }
        // SAFETY: `notification_port` was returned by IOKit and produces a
        // valid run-loop source while the notification port remains alive.
        let Some(source) = (unsafe { IONotificationPort::run_loop_source(notification_port) })
        else {
            return;
        };
        let Some(run_loop) = CFRunLoop::current() else {
            return;
        };
        // SAFETY: Core Foundation provides this global run-loop mode constant
        // for process-wide read-only use.
        let default_mode = unsafe { kCFRunLoopDefaultMode };
        run_loop.add_source(Some(&source), default_mode);
        CFRunLoop::run();
    }

    unsafe extern "C-unwind" fn sleep_callback(
        refcon: *mut c_void,
        _service: io_service_t,
        message_type: u32,
        message_argument: *mut c_void,
    ) {
        // SAFETY: `refcon` is the `CallbackContext` pointer registered with
        // `IORegisterForSystemPower`; it remains allocated while this callback
        // can be invoked by the watcher run loop.
        let context = unsafe { &*(refcon.cast::<CallbackContext>()) };
        match message_type {
            event if event == kIOMessageCanSystemSleep => {
                IOAllowPowerChange(context.root_port, message_argument as isize);
            }
            event if event == kIOMessageSystemWillSleep => {
                if let Ok(mut handler) = context.handler.lock() {
                    handler(SleepEvent::SuspendRequested);
                }
                IOAllowPowerChange(context.root_port, message_argument as isize);
            }
            event if event == kIOMessageSystemHasPoweredOn => {
                if let Ok(mut handler) = context.handler.lock() {
                    handler(SleepEvent::Resumed);
                }
            }
            _ => {}
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::ffi::c_void;
    use windows_sys::Win32::System::Power::{
        RegisterSuspendResumeNotification, SetThreadExecutionState,
        UnregisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, ES_CONTINUOUS,
        ES_SYSTEM_REQUIRED,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
    };

    pub(super) fn spawn(sender: Sender<SleepEvent>) -> io::Result<()> {
        thread::Builder::new()
            .name("lockbox-sleep-watcher".to_string())
            .spawn(move || watch_power_notifications(sender))
            .map(|_| ())
    }

    pub(super) fn agent_sleep_support() -> AgentSleepSupport {
        AgentSleepSupport {
            suspend_notifications: true,
            sleep_inhibition: true,
        }
    }

    pub(super) struct SleepInhibitor;

    impl SleepInhibitor {
        pub(super) fn acquire_active(_reason: &str) -> io::Result<Self> {
            // SAFETY: `SetThreadExecutionState` has no Rust-side memory
            // invariants and stores process/thread execution-state flags.
            let previous = unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED) };
            if previous == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(Self)
            }
        }
    }

    impl Drop for SleepInhibitor {
        fn drop(&mut self) {
            // SAFETY: Restores the continuous execution state flag when the
            // inhibitor guard is dropped.
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    }

    fn watch_power_notifications(sender: Sender<SleepEvent>) {
        let sender = Box::new(sender);
        let context = Box::into_raw(sender);
        let mut params = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(power_callback),
            Context: context.cast::<c_void>(),
        };
        // SAFETY: `params` points to a valid subscription record while the
        // watcher thread parks below. The callback context is a boxed Sender
        // that also lives for the process lifetime of the agent.
        let registration = unsafe {
            RegisterSuspendResumeNotification(
                (&mut params as *mut DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS).cast(),
                DEVICE_NOTIFY_CALLBACK,
            )
        };
        if registration == 0 {
            // SAFETY: Reclaims the boxed sender when registration failed.
            unsafe {
                drop(Box::from_raw(context));
            }
            return;
        }
        loop {
            thread::park();
        }
        // SAFETY: This cleanup is unreachable while the callback registration
        // is active. If the loop gains an exit path, `registration` and
        // `context` are still the owned values created above and must each be
        // released exactly once here.
        #[allow(unreachable_code)]
        unsafe {
            UnregisterSuspendResumeNotification(registration);
            drop(Box::from_raw(context));
        }
    }

    unsafe extern "system" fn power_callback(
        context: *const c_void,
        event_type: u32,
        _setting: *const c_void,
    ) -> u32 {
        if context.is_null() {
            return 0;
        }
        // SAFETY: `context` is the boxed `Sender<SleepEvent>` registered with
        // `RegisterSuspendResumeNotification` and remains allocated for the
        // lifetime of the notification callback.
        let sender = unsafe { &*(context.cast::<Sender<SleepEvent>>()) };
        match event_type {
            PBT_APMSUSPEND => {
                let _ = sender.send(SleepEvent::SuspendRequested);
            }
            PBT_APMRESUMESUSPEND | PBT_APMRESUMEAUTOMATIC => {
                let _ = sender.send(SleepEvent::Resumed);
            }
            _ => {}
        }
        0
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod platform {
    use super::*;

    pub(super) fn agent_sleep_support() -> AgentSleepSupport {
        AgentSleepSupport {
            suspend_notifications: false,
            sleep_inhibition: false,
        }
    }

    pub(super) struct SleepInhibitor;

    impl SleepInhibitor {
        pub(super) fn acquire_active(_reason: &str) -> io::Result<Self> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "sleep inhibition is not supported on this platform",
            ))
        }
    }

    #[cfg(unix)]
    pub(super) fn spawn_handler(_handler: SleepHandler) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "sleep notifications are not supported on this platform",
        ))
    }
}
