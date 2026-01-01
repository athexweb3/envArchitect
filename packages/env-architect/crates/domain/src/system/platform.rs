use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OsType {
    MacOS,
    Linux,
    Windows,
    FreeBSD,
    OpenBSD,
    Unknown,
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OsType::MacOS => write!(f, "macOS"),
            OsType::Linux => write!(f, "Linux"),
            OsType::Windows => write!(f, "Windows"),
            OsType::FreeBSD => write!(f, "FreeBSD"),
            OsType::OpenBSD => write!(f, "OpenBSD"),
            OsType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Arm,
    I686,
    Unknown,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Arm => write!(f, "arm"),
            Architecture::I686 => write!(f, "i686"),
            Architecture::Unknown => write!(f, "unknown"),
        }
    }
}

/// Complete platform information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os_type: OsType,
    pub os_version: String,
    pub arch: Architecture,
    pub distro: Option<String>, // For Linux distributions
    pub kernel_version: Option<String>,
}

impl fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({})", self.os_type, self.os_version, self.arch)?;
        if let Some(distro) = &self.distro {
            write!(f, " [{}]", distro)?;
        }
        Ok(())
    }
}

/// Platform detection service
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detect the current platform information
    pub fn detect() -> PlatformInfo {
        let info = os_info::get();

        let os_type = Self::detect_os_type(&info);
        let arch = Self::detect_architecture();
        let distro = Self::detect_distro(&info, os_type);

        PlatformInfo {
            os_type,
            os_version: info.version().to_string(),
            arch,
            distro,
            kernel_version: sysinfo::System::kernel_version(),
        }
    }

    fn detect_os_type(info: &os_info::Info) -> OsType {
        match info.os_type() {
            os_info::Type::Macos => OsType::MacOS,
            os_info::Type::Windows => OsType::Windows,
            os_info::Type::Alpine
            | os_info::Type::Arch
            | os_info::Type::CentOS
            | os_info::Type::Debian
            | os_info::Type::Fedora
            | os_info::Type::Linux
            | os_info::Type::Mint
            | os_info::Type::NixOS
            | os_info::Type::openSUSE
            | os_info::Type::OracleLinux
            | os_info::Type::Pop
            | os_info::Type::Raspbian
            | os_info::Type::Redhat
            | os_info::Type::RedHatEnterprise
            | os_info::Type::Solus
            | os_info::Type::Ubuntu => OsType::Linux,
            os_info::Type::FreeBSD => OsType::FreeBSD,
            os_info::Type::OpenBSD => OsType::OpenBSD,
            _ => OsType::Unknown,
        }
    }

    fn detect_architecture() -> Architecture {
        match std::env::consts::ARCH {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            "arm" => Architecture::Arm,
            "x86" | "i686" => Architecture::I686,
            _ => Architecture::Unknown,
        }
    }

    fn detect_distro(info: &os_info::Info, os_type: OsType) -> Option<String> {
        if os_type != OsType::Linux {
            return None;
        }

        // Extract distribution name
        match info.os_type() {
            os_info::Type::Ubuntu => Some("Ubuntu".to_string()),
            os_info::Type::Debian => Some("Debian".to_string()),
            os_info::Type::Fedora => Some("Fedora".to_string()),
            os_info::Type::Arch => Some("Arch Linux".to_string()),
            os_info::Type::CentOS => Some("CentOS".to_string()),
            os_info::Type::RedHatEnterprise => Some("RHEL".to_string()),
            os_info::Type::Mint => Some("Linux Mint".to_string()),
            os_info::Type::Pop => Some("Pop!_OS".to_string()),
            os_info::Type::Alpine => Some("Alpine Linux".to_string()),
            os_info::Type::NixOS => Some("NixOS".to_string()),
            _ => Some("Linux".to_string()),
        }
    }

    /// Check if current platform matches given constraints
    pub fn matches(
        info: &PlatformInfo,
        os_type: Option<OsType>,
        arch: Option<Architecture>,
    ) -> bool {
        if let Some(required_os) = os_type {
            if info.os_type != required_os {
                return false;
            }
        }

        if let Some(required_arch) = arch {
            if info.arch != required_arch {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let info = PlatformDetector::detect();

        // Should always detect something
        assert_ne!(info.os_type, OsType::Unknown);
        assert_ne!(info.arch, Architecture::Unknown);

        println!("Detected platform: {}", info);
    }

    #[test]
    fn test_platform_matching() {
        let info = PlatformDetector::detect();

        // Should match itself
        assert!(PlatformDetector::matches(
            &info,
            Some(info.os_type),
            Some(info.arch)
        ));

        // Should match with no constraints
        assert!(PlatformDetector::matches(&info, None, None));
    }
}
