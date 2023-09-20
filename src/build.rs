use crate::config::{Arch, Config};
use crate::make::MakeCmd;
use crate::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tracing::*;

#[instrument(name = "build", level = "debug", skip(config))]
pub fn build<I, S>(config: &Config, args: I) -> Result<String>
where
    I: IntoIterator<Item = S> + core::fmt::Debug + Clone,
    S: AsRef<std::ffi::OsStr>,
{
    // TODO: backup old config if precise

    let config_file = crate::kconfig::new_config(config, args.clone())?;

    // run olddefconfig
    MakeCmd::new(config, Some("olddefconfig"), args.clone())?.run()?;

    crate::kconfig::check_configs(config, &config_file)?;

    MakeCmd::new(config, config.make.make_arch_target(), args.clone())?.run()?;

    let mut boot = config.make.make_build_dir();
    boot.push("arch");
    boot.push(config.make.arch.context("Arch missing")?.kernel_arch());
    boot.push("boot");
    let out = config.make.kernel_bin_dir();

    match config.make.arch.context("Arch missing")? {
        Arch::X86 | Arch::X86_64 => {
            install_file(boot.join("bzImage"), out.join("vmlinuz"))?;
        }
        Arch::Aarch64 => {
            install_file(boot.join("Image"), out.join("vmlinuz"))?;
        }
        Arch::Mips => {
            install_file(boot.join("vmlinux.strip"), out.join("vmlinuz"))?;
        }
        _ => {
            todo!();
        }
    }

    install_file(
        config.make.make_build_dir().join("vmlinux"),
        out.join("vmlinux"),
    )?;
    install_file(
        config.make.make_build_dir().join(".config"),
        out.join("config"),
    )?;

    // if there weren't actually any modules selected
    drop(std::fs::File::create(
        config.make.make_build_dir().join("modules.order"),
    ));
    drop(std::fs::File::create(
        config.make.make_build_dir().join("modules.builtin"),
    ));

    drop(MakeCmd::new(config, Some("modules_install"), args.clone())?.run());

    let version = std::fs::read_to_string(
        config
            .make
            .make_build_dir()
            .join("include/config/kernel.release"),
    )
    .map(|s| s.trim().to_string())
    .context("Failed to read kernel release")?;
    info!("Installed kernel version {}", version);

    // TODO: depmod?
    Ok(version)
}

fn install_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result {
    install_file_perm(src, dst, std::fs::Permissions::from_mode(0o644))
}

fn install_file_perm(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    perm: std::fs::Permissions,
) -> Result {
    //trace!(src = src.into().display(), dst = dst.into().display(), perm = format!("{perm:?}"), "installing file");
    trace!(
        "Installing file {} to {} with permissions {:?}",
        src.as_ref().display(),
        dst.as_ref().display(),
        perm
    );

    if let Some(parent) = dst.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(&src, &dst)?;
    std::fs::set_permissions(&dst, perm)?;

    Ok(())
}
