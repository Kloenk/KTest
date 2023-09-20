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

pub fn get_test_deps(_config: &Config, test: impl AsRef<std::ffi::OsStr>) -> Result<String> {
    let mut cmd = Command::new(test);
    cmd.arg("deps").env("KTEST_TEST_LIB", "./lib/testlib.sh");

    debug!(
        "Running {} {}",
        cmd.get_program()
            .to_str()
            .unwrap_or("**could not find executable**"),
        cmd.get_args()
            .filter_map(|a| a.to_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let out = cmd.output()?;

    if !out.status.success() {
        info!("Failed to run test binary: {}", out.status);
        return Err(Error::new("Failed to run test in dep mode").set_exit_code(out.status.code()));
    }

    let stdout = String::from_utf8(out.stdout).map_err(|e| Error::new(e.to_string()))?;
    Ok(stdout)
}

pub fn update_config_for_test(config: &mut Config, test: impl AsRef<std::ffi::OsStr>) -> Result {
    let string = get_test_deps(config, test)?;

    for line in string.lines() {
        let line = line.splitn(2, '=').collect::<Vec<_>>();
        let key = line[0];
        let value = line[1];

        debug!(key, value, "Parsing value");

        match key {
            "ktest_arch" => config.make.arch = Some(value.parse()?),
            "ktest_cpus" => config.qemu.cpus = value.parse()?,
            "ktest_mem" => config.qemu.mem = value.to_string(),
            "ktest_kernel_append" => config
                .qemu
                .extra_kernel_args
                .extend(parse_array::<String>(value)?),
            "ktest_kernel_make_append" => config
                .make
                .extra_make_args
                .extend(parse_array::<String>(value)?),
            "ktest_kernel_config_require" => {
                let cfg = parse_array::<String>(value)?;
                for cfg in &cfg {
                    let (key, value) = crate::kconfig::parse(cfg)?;
                    config.make.kconfig.insert(key, value);
                }
            }
            _ => warn!(key, value, "Unknown key"),
        }
    }

    Ok(())
}

fn parse_array<T>(value: &str) -> Result<Vec<T>>
where
    T: FromStr,
    Error: From<<T as FromStr>::Err>,
{
    let mut out = Vec::new();
    let value = value.trim_start_matches('(').trim_end_matches(')');
    for item in value.split(' ') {
        out.push(item.parse()?);
    }
    Ok(out)
}
