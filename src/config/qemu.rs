use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Qemu {
    pub path_override: Option<String>,
    #[serde(flatten)]
    pub arch_config: HashMap<super::make::Arch, QemuConfig>,
}

impl Qemu {
    pub fn path(&self, arch: &super::make::Arch) -> Option<&str> {
        self.path_override
            .as_ref()
            .or_else(|| self.arch_config.get(arch).map(|c| &c.path))
            .map(String::as_str)
    }

    /*pub fn args(&self) -> Vec<Arg> {
        let mut ret = Vec::new();

        ret.push(
            Arg::new("qemu-path")
                .long("qemu-path")
                .action(ArgAction::Set)
                .value_name("PATH")
                .value_hint(ValueHint::ExecutablePath)
                .default_value(self.path.clone())
                .hide(true),
        );

        ret
    }

    pub fn group() -> ArgGroup {
        ArgGroup::new("qemu")
            //.args(["qemu-path"])
            .multiple(true)
    }*/
}

#[derive(Debug, Clone, Deserialize)]
pub struct QemuConfig {
    pub path: String,
    //pub args: Vec<String>,
}
