use crate::config::Config;
use crate::{Context, Error, Result};
use std::process::Command;
use std::str::FromStr;
use tracing::*;

pub struct QemuCmd {
    pub cmd: Command,
}

impl QemuCmd {
    pub fn new(config: &Config) -> Result<Self> {
        Self::setup_dirs(config)?;

        let mut cmd = Command::new(config.qemu_path().context("Missing qemu executable")?);
        cmd.args(config.qemu_args());

        let mut kernel_args: Vec<String> =
            config.qemu_kernel_args().map(|s| s.to_string()).collect();
        match config.qemu.storage_bus.as_str() {
            "virtio-blk" => kernel_args.push("root=/dev/vda".to_string()),
            _ => kernel_args.push("root=/dev/sda".to_string()),
        }

        cmd.arg("-m").arg(format!(
            "{mem},slots=8,maxmem={maxmem}",
            mem = config.qemu.mem(),
            maxmem = maxmem(),
        ));
        cmd.arg("-smp")
            .arg(format!("{cpus}", cpus = config.qemu.cpus()));
        cmd.arg("-kernel")
            .arg(config.make.kernel_bin_dir().join("vmlinuz"));
        cmd.arg("-append").arg(kernel_args.join(" "));
        cmd.arg("-serial").arg(format!(
            "unix:{},server,nowait",
            config.make.out_dir().join("vm").join("kgdb").display()
        ));
        cmd.arg("-monitor").arg(format!(
            "unix:{},server,nowait",
            config.make.out_dir().join("vm").join("mon").display()
        ));
        cmd.arg("-gdb").arg(format!(
            "unix:{},server,nowait",
            config.make.out_dir().join("vm").join("gdb").display()
        ));

        Ok(Self { cmd })
    }

    fn setup_dirs(config: &Config) -> Result {
        // TODO: remove old dirs

        std::fs::create_dir_all(config.make.out_dir().join("vm"))
            .context("Failed to create vm socket dir")?;

        Ok(())
    }

    pub fn run(&mut self) -> Result {
        debug!(
            "Running {} {}",
            self.cmd
                .get_program()
                .to_str()
                .unwrap_or("**could not find executable**"),
            self.cmd
                .get_args()
                .filter_map(|a| a.to_str())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let status = self.cmd.status().context("Failed to run qemu")?;

        if !status.success() {
            info!("Failed to run qemu: {}", status);
            Err(Error::new("Failed to run qemu").set_exit_code(status.code()))
        } else {
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
pub fn maxmem() -> String {
    let maxmem = procfs::Meminfo::new()
        .map(|m| m.mem_total)
        .unwrap_or(1024 * 1024 * 1024);
    let maxmem = "3G";
    println!("todo");
    "3G".to_string()
}

#[cfg(target_os = "macos")]
pub fn maxmem() -> String {
    println!("todo");
    "3G".to_string()
}
