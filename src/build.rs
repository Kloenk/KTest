use crate::config::Config;
use crate::make::MakeCmd;
use crate::Result;
use tracing::*;

#[instrument(name = "build", level = "debug", skip(config))]
pub fn build<I, S>(config: &Config, args: I) -> Result
where
    I: IntoIterator<Item = S> + core::fmt::Debug + Clone,
    S: AsRef<std::ffi::OsStr>,
{
    // TODO: backup old config if precise

    let config_file = crate::kconfig::new_config(config, args.clone())?;

    // run olddefconfig
    MakeCmd::new(config, Some("olddefconfig"), args.clone())?.run()?;

    crate::kconfig::check_configs(config, &config_file)?;

    MakeCmd::new(config, config.make.make_arch_target(), args)?.run()?;

    Ok(())
}
