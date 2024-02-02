use std::fmt::{Display, Formatter};

#[allow(clippy::enum_variant_names)]
#[derive(PartialEq, Eq)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

impl Display for OS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OS::Windows => "windows".to_string(),
            OS::Linux => "linux".to_string(),
            OS::MacOS => "macos".to_string(),
        };
        write!(f, "{}", str)
    }
}