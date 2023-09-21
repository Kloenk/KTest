use ktest_core::config::{Arch, Config, KConfig};
use ktest_core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use tracing::*;

#[derive(Debug, Default)]
pub struct Deps {
    arch: Option<Arch>,
    cpus: Option<usize>,
    mem: Option<String>,
    kernel_append: Vec<String>,
    make_append: Vec<String>,
    storage_bus: Option<String>,
    config: Vec<KConfig>,
    qemu_append: Vec<String>,
}

impl Deps {
    #[instrument(level = "debug", skip(config))]
    pub fn get(config: &Config, test: &str) -> Result<Self> {
        // TODO: compare test with builtin tests

        Self::external(config, Path::new(test))
    }

    pub fn external(config: &Config, test: &Path) -> Result<Self> {
        let test = test.canonicalize()?;
        if !test.exists() {
            return Err(format!("test {:?} does not exist", test).into());
        }

        match test.extension().map(|s| s.to_str()).flatten() {
            // TODO: scm for steel test
            Some("ktest") | _ => Self::external_str(config, test),
        }
    }

    fn external_str(config: &Config, test: PathBuf) -> Result<Self> {
        let string = Self::external_str_run(config, test)?;

        let mut out = Self::default();

        for line in string.lines() {
            let line = line.splitn(2, '=').collect::<Vec<_>>();
            let key = line[0];
            let value = line[1];

            debug!(key, value, "Parsing value");

            match key {
                "ktest_arch" => out.arch = Some(value.parse()?),
                "ktest_cpus" => out.cpus = Some(value.parse()?),
                "ktest_mem" => out.mem = Some(value.to_string()),
                "ktest_kernel_append" => out.kernel_append.extend(parse_array::<String>(value)?),
                "ktest_kernel_make_append" => out.make_append.extend(parse_array::<String>(value)?),
                "ktest_storage_bus" => out.storage_bus = Some(value.to_string()),
                "ktest_kernel_config_require" => {
                    let cfg = parse_array::<String>(value)?;
                    for cfg in cfg {
                        out.config.push(cfg.parse()?);
                    }
                }
                "ktest_qemu_append" => out.qemu_append.extend(parse_array::<String>(value)?),
                _ => warn!(key, value, "Unknown key"),
            }
        }

        Ok(out)
    }
    fn external_str_run(config: &Config, test: PathBuf) -> Result<String> {
        let mut cmd = Command::new(test);
        cmd.arg("deps")
            .env("KTEST_TEST_LIB", config.ktest.lib_path()?)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

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
            return Err(
                Error::new("Failed to run test in deps mode").set_exit_code(out.status.code())
            );
        }

        let stdout = String::from_utf8(out.stdout).map_err(|e| Error::new(e.to_string()))?;
        Ok(stdout)
    }
}

impl Deps {
    pub fn merge_into_config(&self, config: &mut Config) {
        if let Some(arch) = &self.arch {
            config.make.arch = Some(*arch);
        }
        if let Some(cpus) = &self.cpus {
            config.qemu.cpus = *cpus;
        }
        if let Some(mem) = &self.mem {
            config.qemu.mem = mem.clone();
        }
        if !self.kernel_append.is_empty() {
            config
                .qemu
                .extra_kernel_args
                .extend(self.kernel_append.clone());
        }
        if !self.make_append.is_empty() {
            config.make.extra_make_args.extend(self.make_append.clone());
        }
        if let Some(storage_bus) = &self.storage_bus {
            config.qemu.storage_bus = storage_bus.clone();
        }
        if !self.config.is_empty() {
            for cfg in &self.config {
                config.make.kconfig.insert(cfg.key.clone(), cfg.clone());
            }
        }
        if !self.qemu_append.is_empty() {
            config.qemu.extra_args.extend(self.qemu_append.clone());
        }
    }
    pub fn print_shell(&self) {
        if let Some(arch) = &self.arch {
            println!("ktest_arch={}", arch);
        }
        if let Some(cpus) = &self.cpus {
            println!("ktest_cpus={}", cpus);
        }
        if let Some(mem) = &self.mem {
            println!("ktest_mem={}", mem);
        }
        if !self.kernel_append.is_empty() {
            println!("ktest_kernel_append=({})", self.kernel_append.join(" "));
        }
        if !self.make_append.is_empty() {
            println!("ktest_kernel_make_append=({})", self.make_append.join(" "));
        }
        if let Some(storage_bus) = &self.storage_bus {
            println!("ktest_storage_bus={}", storage_bus);
        }
        if !self.config.is_empty() {
            println!(
                "ktest_kernel_config_require=({})",
                self.config
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        if !self.qemu_append.is_empty() {
            println!("ktest_qemu_append=({})", self.qemu_append.join(" "));
        }
    }
}

fn parse_array<T>(value: &str) -> Result<Vec<T>>
where
    T: FromStr,
    Error: From<<T as FromStr>::Err>,
{
    let mut out = Vec::new();
    let value = value.trim_start_matches('(').trim_end_matches(')');
    for item in value.split(' ') {
        if item.is_empty() {
            continue;
        }
        out.push(item.parse()?);
    }
    Ok(out)
}
