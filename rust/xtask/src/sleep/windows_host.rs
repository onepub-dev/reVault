use crate::command::{self, TaskResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const WINDOWS_TARGET: &str = "x86_64-pc-windows-gnu";

struct Options {
    domain: String,
    disk: PathBuf,
    xml: PathBuf,
    nbd: String,
    mount_dir: PathBuf,
    out_dir: PathBuf,
    skip_build: bool,
    skip_task_check: bool,
    prepare_only: bool,
    extract_only: bool,
    force_destroy: bool,
    recover_ntfs: bool,
}

struct Runner {
    options: Options,
    root: PathBuf,
    connected: bool,
}

pub fn run(args: &[String]) -> TaskResult {
    let Some(options) = parse(args)? else {
        return Ok(());
    };
    let root = command::workspace_root()?;
    let mut runner = Runner {
        options,
        root,
        connected: false,
    };
    runner.preflight()?;
    if runner.options.extract_only {
        return runner.extract_results();
    }
    runner.stage_iso()?;
    runner.prepare_disk()?;
    if runner.options.prepare_only {
        println!("prepared Windows disk and transfer ISO");
        return Ok(());
    }
    runner.run_domain_test()?;
    runner.extract_results()
}

impl Runner {
    fn preflight(&self) -> TaskResult {
        command::require_commands(&[
            "cargo",
            "genisoimage",
            "findmnt",
            "lsblk",
            "ntfs-3g",
            "qemu-nbd",
            "virsh",
            "sudo",
        ])?;
        if self.options.recover_ntfs {
            command::require_commands(&["ntfsfix"])?;
        }
        command::require_file(&self.options.disk)?;
        command::require_file(&self.options.xml)?;
        command::run(command::command("sudo").arg("-v"))
    }

    fn stage_iso(&self) -> TaskResult {
        if !self.options.skip_build {
            println!("stage: build Windows lockbox.exe and xtask.exe");
            command::run(command::command("cargo").args([
                "build",
                "--manifest-path",
                "revault_cli/Cargo.toml",
                "--target",
                WINDOWS_TARGET,
            ]))?;
            command::run(command::command("cargo").args([
                "build",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--target",
                WINDOWS_TARGET,
            ]))?;
        }
        let target = self.root.join("target").join(WINDOWS_TARGET).join("debug");
        let lockbox = target.join("lockbox.exe");
        let xtask = target.join("xtask.exe");
        command::require_file(&lockbox)?;
        command::require_file(&xtask)?;
        let vm_dir = self.root.join("target/vm/windows");
        let share = vm_dir.join("share");
        fs::create_dir_all(&share)
            .map_err(|error| format!("cannot create {}: {error}", share.display()))?;
        fs::copy(lockbox, share.join("LOCKBOX.EXE"))
            .map_err(|error| format!("cannot stage lockbox.exe: {error}"))?;
        fs::copy(xtask, share.join("XTASK.EXE"))
            .map_err(|error| format!("cannot stage xtask.exe: {error}"))?;
        let iso = vm_dir.join("lockbox-transfer.iso");
        println!("stage: create transfer ISO");
        command::run(command::command("genisoimage").args([
            "-quiet",
            "-V",
            "LOCKBOX",
            "-r",
            "-J",
            "-o",
            &iso.to_string_lossy(),
            &share.to_string_lossy(),
        ]))
    }

    fn connect_disk(&mut self, read_only: bool) -> TaskResult {
        println!(
            "stage: connect {} to {}",
            self.options.disk.display(),
            self.options.nbd
        );
        command::run(command::command("sudo").args(["modprobe", "nbd", "max_part=16"]))?;
        let mut qemu = command::command("sudo");
        qemu.args(["qemu-nbd", &format!("--connect={}", self.options.nbd)]);
        if read_only {
            qemu.arg("--read-only");
        }
        qemu.arg(&self.options.disk);
        command::run(&mut qemu)?;
        self.connected = true;
        thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    fn ntfs_partition(&self) -> TaskResult<String> {
        let listing = command::output_lossy(command::command("lsblk").args([
            "-nrpo",
            "NAME,FSTYPE",
            &self.options.nbd,
        ]))?;
        listing
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let name = fields.next()?;
                (fields.next() == Some("ntfs")).then(|| name.to_owned())
            })
            .next()
            .ok_or_else(|| format!("could not find an NTFS partition on {}", self.options.nbd))
    }

    fn mount_disk(&self, read_only: bool) -> TaskResult {
        let partition = self.ntfs_partition()?;
        println!("stage: mount {partition}");
        command::run(command::command("sudo").args([
            "mkdir",
            "-p",
            &self.options.mount_dir.to_string_lossy(),
        ]))?;
        if read_only {
            return command::run(command::command("sudo").args([
                "mount",
                "-o",
                "ro",
                &partition,
                &self.options.mount_dir.to_string_lossy(),
            ]));
        }
        let normal = command::command("sudo")
            .args([
                "ntfs-3g",
                "-o",
                "windows_names",
                &partition,
                &self.options.mount_dir.to_string_lossy(),
            ])
            .status();
        if normal.is_ok_and(|status| status.success()) && self.mount_is_rw() {
            return Ok(());
        }
        self.unmount();
        if !self.options.recover_ntfs {
            return Err("NTFS could not be mounted read/write; rerun with --recover-ntfs only for the disposable copied qcow2 disk".to_owned());
        }
        println!("stage: recover NTFS dirty or hibernated state");
        command::run(command::command("sudo").args(["ntfsfix", "-d", &partition]))?;
        command::run(command::command("sudo").args([
            "ntfs-3g",
            "-o",
            "windows_names,remove_hiberfile",
            &partition,
            &self.options.mount_dir.to_string_lossy(),
        ]))?;
        if self.mount_is_rw() {
            Ok(())
        } else {
            Err("NTFS mount is still read-only; cannot write C:\\lhs.cmd".to_owned())
        }
    }

    fn mount_is_rw(&self) -> bool {
        let options = command::output_lossy(command::command("findmnt").args([
            "-n",
            "-o",
            "OPTIONS",
            "--target",
            &self.options.mount_dir.to_string_lossy(),
        ]))
        .unwrap_or_default();
        !options.is_empty() && !options.split(',').any(|option| option == "ro")
    }

    fn write_boot_command(&self) -> TaskResult {
        println!("stage: write C:\\lhs.cmd");
        let contents = r#"@echo off
set LOG=C:\lockbox-agent-sleep-test.txt
echo started %DATE% %TIME% > %LOG%
echo powercfg /a before test: >> %LOG%
powercfg /a >> %LOG% 2>&1
echo running lockbox sleep test >> %LOG%
D:\XTASK.EXE agent-sleep-windows-vm --bin D:\LOCKBOX.EXE >> %LOG% 2>&1
echo test exit=%ERRORLEVEL% >> %LOG%
echo powercfg /a after test: >> %LOG%
powercfg /a >> %LOG% 2>&1
schtasks /delete /tn lhs /f >> %LOG% 2>&1
shutdown /s /t 0 /f
"#;
        let temporary =
            std::env::temp_dir().join(format!("revault-lhs-{}.cmd", std::process::id()));
        fs::write(&temporary, contents)
            .map_err(|error| format!("cannot write temporary lhs.cmd: {error}"))?;
        let destination = self.options.mount_dir.join("lhs.cmd");
        let result = command::run(command::command("sudo").args([
            "install",
            "-m",
            "0644",
            &temporary.to_string_lossy(),
            &destination.to_string_lossy(),
        ]));
        let _ = fs::remove_file(temporary);
        result?;
        let task = self.options.mount_dir.join("Windows/System32/Tasks/lhs");
        if !self.options.skip_task_check && !task.is_file() {
            return Err("C:\\Windows\\System32\\Tasks\\lhs is missing; run `cargo xtask agent-sleep-windows-setup --force-destroy --launch-manager`".to_owned());
        }
        Ok(())
    }

    fn prepare_disk(&mut self) -> TaskResult {
        self.connect_disk(false)?;
        self.mount_disk(false)?;
        self.write_boot_command()?;
        command::run(command::command("sudo").arg("sync"))?;
        self.cleanup();
        Ok(())
    }

    fn domain_state(&self) -> String {
        command::output_lossy(virsh().args(["domstate", &self.options.domain])).unwrap_or_default()
    }

    fn ensure_domain_ready(&self) -> TaskResult {
        let mut state = self.domain_state();
        if matches!(state.as_str(), "running" | "paused" | "pmsuspended") {
            if !self.options.force_destroy {
                return Err(format!("domain {} is already active: {state}; use --force-destroy for the disposable test VM", self.options.domain));
            }
            println!("stage: destroy active test domain");
            command::run(virsh().args(["destroy", &self.options.domain]))?;
            for _ in 0..60 {
                state = self.domain_state();
                if state == "shut off" || state.is_empty() {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
        if !self.domain_state().is_empty() {
            let status = virsh()
                .args(["undefine", &self.options.domain, "--nvram"])
                .status();
            if !status.is_ok_and(|status| status.success()) {
                command::run(virsh().args(["undefine", &self.options.domain]))?;
            }
        }
        println!("stage: define libvirt domain");
        command::run(virsh().args(["define", &self.options.xml.to_string_lossy()]))
    }

    fn run_domain_test(&self) -> TaskResult {
        self.ensure_domain_ready()?;
        println!("stage: start headless Windows VM");
        command::run(virsh().args(["start", &self.options.domain]))?;
        println!("stage: wait for suspend");
        for _ in 0..240 {
            let state = self.domain_state();
            if state == "pmsuspended" {
                println!("stage: wake suspended VM");
                let _ = virsh().args(["dompmwakeup", &self.options.domain]).status();
                break;
            }
            if state == "shut off" {
                break;
            }
            thread::sleep(Duration::from_secs(2));
        }
        println!("stage: wait for VM shutdown");
        for _ in 0..240 {
            if self.domain_state() == "shut off" {
                return Ok(());
            }
            thread::sleep(Duration::from_secs(2));
        }
        Err(format!(
            "VM did not shut down after the test; current state: {}",
            self.domain_state()
        ))
    }

    fn extract_results(&mut self) -> TaskResult {
        fs::create_dir_all(&self.options.out_dir).map_err(|error| {
            format!("cannot create {}: {error}", self.options.out_dir.display())
        })?;
        self.connect_disk(true)?;
        self.mount_disk(true)?;
        println!("stage: extract result files");
        let result = self
            .copy_as_user(
                "lockbox-agent-sleep-test.txt",
                "lockbox-agent-sleep-test.txt",
            )?
            .ok_or_else(|| "missing result file: C:\\lockbox-agent-sleep-test.txt".to_owned())?;
        let agent_log = self.copy_as_user(
            "Windows/Temp/lockbox-agent-sleep-test/agent.log",
            "agent.log",
        )?;
        print_file("result", &result)?;
        if let Some(agent_log) = &agent_log {
            print_file("agent log", agent_log)?;
        } else {
            eprintln!("missing agent log: C:\\Windows\\Temp\\lockbox-agent-sleep-test\\agent.log");
        }
        let result_text = fs::read_to_string(&result).map_err(|error| error.to_string())?;
        if !result_text.contains("pass: cache cleared on sleep") {
            return Err("Windows sleep test did not report success".to_owned());
        }
        if let Some(agent_log) = agent_log {
            let log = fs::read_to_string(agent_log).map_err(|error| error.to_string())?;
            if !log.contains("suspend requested; cleared") {
                return Err("agent log did not show suspend cache clearing".to_owned());
            }
        }
        println!("pass: Windows agent sleep test completed");
        Ok(())
    }

    fn copy_as_user(&self, source: &str, destination: &str) -> TaskResult<Option<PathBuf>> {
        let source = self.options.mount_dir.join(source);
        if !source.is_file() {
            return Ok(None);
        }
        let contents =
            command::output(command::command("sudo").args(["cat", &source.to_string_lossy()]))?;
        let destination = self.options.out_dir.join(destination);
        fs::write(&destination, contents.stdout)
            .map_err(|error| format!("cannot write {}: {error}", destination.display()))?;
        Ok(Some(destination))
    }

    fn unmount(&self) {
        let mounted = command::command("findmnt")
            .args(["-n", "--target", &self.options.mount_dir.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if mounted {
            let _ = command::command("sudo")
                .args(["umount", &self.options.mount_dir.to_string_lossy()])
                .status();
        }
    }

    fn cleanup(&mut self) {
        self.unmount();
        if self.connected {
            let _ = command::command("sudo")
                .args(["qemu-nbd", "--disconnect", &self.options.nbd])
                .status();
            self.connected = false;
        }
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn virsh() -> Command {
    let mut command = command::command("virsh");
    command.args(["-c", "qemu:///session"]);
    command
}

fn print_file(label: &str, path: &Path) -> TaskResult {
    println!("{label}: {}", path.display());
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    for line in text.lines().take(220) {
        println!("{line}");
    }
    Ok(())
}

fn parse(args: &[String]) -> TaskResult<Option<Options>> {
    let root = command::workspace_root()?;
    let vm = root.join("target/vm/windows");
    let mut options = Options {
        domain: "lockbox-win-s3-test".to_owned(),
        disk: vm.join("windows-server-2025-headless-task.qcow2"),
        xml: vm.join("lockbox-win-s3-libvirt.xml"),
        nbd: "/dev/nbd0".to_owned(),
        mount_dir: PathBuf::from("/tmp/lockbox-win-s3-mount"),
        out_dir: vm.join("results"),
        skip_build: false,
        skip_task_check: false,
        prepare_only: false,
        extract_only: false,
        force_destroy: false,
        recover_ntfs: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--domain" => options.domain = command::option_value(args, &mut index, "--domain")?,
            "--disk" => {
                options.disk = PathBuf::from(command::option_value(args, &mut index, "--disk")?)
            }
            "--xml" => {
                options.xml = PathBuf::from(command::option_value(args, &mut index, "--xml")?)
            }
            "--nbd" => options.nbd = command::option_value(args, &mut index, "--nbd")?,
            "--mount-dir" => {
                options.mount_dir =
                    PathBuf::from(command::option_value(args, &mut index, "--mount-dir")?)
            }
            "--out-dir" => {
                options.out_dir =
                    PathBuf::from(command::option_value(args, &mut index, "--out-dir")?)
            }
            "--skip-build" => options.skip_build = true,
            "--skip-task-check" => options.skip_task_check = true,
            "--prepare-only" => options.prepare_only = true,
            "--extract-only" => options.extract_only = true,
            "--force-destroy" => options.force_destroy = true,
            "--recover-ntfs" => options.recover_ntfs = true,
            "-h" | "--help" => {
                println!("Usage: cargo xtask agent-sleep-windows-host [--domain NAME] [--disk PATH] [--xml PATH] [--nbd DEVICE] [--mount-dir DIR] [--out-dir DIR] [--skip-build] [--skip-task-check] [--prepare-only] [--extract-only] [--force-destroy] [--recover-ntfs]");
                return Ok(None);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    Ok(Some(options))
}
