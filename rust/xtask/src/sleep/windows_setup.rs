use crate::command::{self, TaskResult};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Options {
    domain: String,
    disk: PathBuf,
    iso: PathBuf,
    force_destroy: bool,
    launch_manager: bool,
}

pub fn run(args: &[String]) -> TaskResult {
    let Some(options) = parse(args)? else {
        return Ok(());
    };
    let root = command::workspace_root()?;
    let vm_dir = root.join("target/vm/windows");
    let vars = vm_dir.join("lockbox-win-task-setup-vars.fd");
    let xml = vm_dir.join("lockbox-win-task-setup.xml");
    let code_template = PathBuf::from("/usr/share/OVMF/OVMF_CODE_4M.fd");
    let vars_template = PathBuf::from("/usr/share/OVMF/OVMF_VARS_4M.fd");
    command::require_commands(&["virsh"])?;
    for file in [&options.disk, &options.iso, &code_template, &vars_template] {
        command::require_file(file)?;
    }
    fs::create_dir_all(&vm_dir)
        .map_err(|error| format!("cannot create {}: {error}", vm_dir.display()))?;

    let state = domain_state(&options.domain);
    if matches!(state.as_str(), "running" | "paused" | "pmsuspended") {
        if !options.force_destroy {
            return Err(format!(
                "domain {} is already active: {state}; rerun with --force-destroy if this is the disposable setup VM",
                options.domain
            ));
        }
        command::run(virsh().args(["destroy", &options.domain]))?;
    }
    if !vars.exists() {
        fs::copy(&vars_template, &vars)
            .map_err(|error| format!("cannot copy UEFI vars: {error}"))?;
    }

    let definition = format!(
        r#"<domain type='kvm'>
  <name>{domain}</name>
  <memory unit='MiB'>4096</memory>
  <currentMemory unit='MiB'>4096</currentMemory>
  <vcpu placement='static'>4</vcpu>
  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <loader readonly='yes' type='pflash'>{code}</loader>
    <nvram template='{vars_template}'>{vars}</nvram>
    <boot dev='hd'/>
  </os>
  <features><acpi/><apic/><smm state='on'/><vmport state='off'/></features>
  <cpu mode='host-passthrough' check='none'/>
  <clock offset='localtime'/>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='file' device='disk'><driver name='qemu' type='qcow2'/><source file='{disk}'/><target dev='sda' bus='sata'/></disk>
    <disk type='file' device='cdrom'><driver name='qemu' type='raw'/><source file='{iso}'/><target dev='sdb' bus='sata'/><readonly/></disk>
    <controller type='sata' index='0'/><controller type='usb' model='qemu-xhci'/>
    <input type='keyboard' bus='usb'/><input type='tablet' bus='usb'/>
    <graphics type='spice' autoport='yes'><listen type='none'/></graphics>
    <video><model type='qxl'/></video><memballoon model='none'/>
  </devices>
</domain>
"#,
        domain = options.domain,
        code = code_template.display(),
        vars_template = vars_template.display(),
        vars = vars.display(),
        disk = options.disk.display(),
        iso = options.iso.display(),
    );
    fs::write(&xml, definition)
        .map_err(|error| format!("cannot write {}: {error}", xml.display()))?;
    command::run(virsh().args(["define", &xml.to_string_lossy()]))?;
    command::run(virsh().args(["start", &options.domain]))?;

    println!("Started temporary visible setup VM: {}", options.domain);
    println!("\nOpen virt-manager, connect to qemu:///session, and open the domain.");
    println!("\nInside Windows, run as Administrator:");
    println!(r"  schtasks /create /tn lhs /tr C:\lhs.cmd /sc onstart /ru SYSTEM /f");
    println!("  shutdown /s /t 0");
    println!("\nThen run:");
    println!("  cargo xtask agent-sleep-windows-host --force-destroy --recover-ntfs");
    if options.launch_manager {
        Command::new("virt-manager")
            .args(["--connect", "qemu:///session"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("cannot launch virt-manager: {error}"))?;
    }
    Ok(())
}

fn virsh() -> Command {
    let mut command = command::command("virsh");
    command.args(["-c", "qemu:///session"]);
    command
}

fn domain_state(domain: &str) -> String {
    command::output_lossy(virsh().args(["domstate", domain])).unwrap_or_default()
}

fn parse(args: &[String]) -> TaskResult<Option<Options>> {
    let root = command::workspace_root()?;
    let vm = root.join("target/vm/windows");
    let mut options = Options {
        domain: "lockbox-win-task-setup".to_owned(),
        disk: vm.join("windows-server-2025-headless-task.qcow2"),
        iso: vm.join("lockbox-transfer.iso"),
        force_destroy: false,
        launch_manager: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--domain" => options.domain = command::option_value(args, &mut index, "--domain")?,
            "--disk" => {
                options.disk = PathBuf::from(command::option_value(args, &mut index, "--disk")?)
            }
            "--iso" => {
                options.iso = PathBuf::from(command::option_value(args, &mut index, "--iso")?)
            }
            "--force-destroy" => options.force_destroy = true,
            "--launch-manager" => options.launch_manager = true,
            "-h" | "--help" => {
                println!("Usage: cargo xtask agent-sleep-windows-setup [--domain NAME] [--disk PATH] [--iso PATH] [--force-destroy] [--launch-manager]");
                return Ok(None);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    Ok(Some(options))
}
