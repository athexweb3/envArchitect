use domain::ports::system::SystemInfo;

pub struct MacOsSystem;

impl SystemInfo for MacOsSystem {
    fn os_name(&self) -> String {
        "macOS".to_string()
    }
    fn arch(&self) -> String {
        std::env::consts::ARCH.to_string()
    }
}
