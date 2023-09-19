use tracing::*;

#[derive(Debug)]
pub struct Error {
    pub anyhow: anyhow::Error,
    pub exit_code: i32,
}

impl Error {
    pub fn anyhow(anyhow: anyhow::Error) -> Self {
        Self {
            anyhow,
            exit_code: 1,
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(anyhow: anyhow::Error) -> Self {
        Self {
            anyhow,
            exit_code: 1,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        trace!("creating error from std::io::Error: {:?}", value);
        let exit_code = value.raw_os_error().map(i32::from).unwrap_or(1);
        let anyhow = anyhow::Error::from(value);

        Self { anyhow, exit_code }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.anyhow.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        //Some(&self.anyhow)
        None
    }
}
