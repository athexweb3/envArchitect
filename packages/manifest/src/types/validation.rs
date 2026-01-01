use crate::{CpuArchitecture, EnhancedManifest, OperatingSystem};
use anyhow::Result;
use schemars::JsonSchema;
use semver::VersionReq;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
pub enum ValidationLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.valid = false;
        self.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            field: field.into(),
            message: message.into(),
        });
    }

    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            level: ValidationLevel::Warning,
            field: field.into(),
            message: message.into(),
        });
    }

    pub fn add_info(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            level: ValidationLevel::Info,
            field: field.into(),
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.level == ValidationLevel::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.level == ValidationLevel::Warning)
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ManifestValidator;

impl ManifestValidator {
    pub fn validate(manifest: &EnhancedManifest) -> ValidationResult {
        let mut result = ValidationResult::new();

        Self::validate_required_fields(manifest, &mut result);
        Self::validate_recommended_fields(manifest, &mut result);
        Self::validate_platform_constraints(manifest, &mut result);
        Self::validate_dependencies(manifest, &mut result);
        Self::validate_profiles(manifest, &mut result);
        Self::validate_groups(manifest, &mut result);

        result
    }

    fn validate_required_fields(manifest: &EnhancedManifest, result: &mut ValidationResult) {
        let total_deps = manifest.dependencies.len()
            + manifest.dev_dependencies.len()
            + manifest.test_dependencies.len()
            + manifest.build_dependencies.len();

        if total_deps == 0 {
            result.add_error(
                "dependencies",
                "At least one dependency is required (dependencies, dev-dependencies, test-dependencies, or build-dependencies)"
            );
        }
    }

    fn validate_recommended_fields(manifest: &EnhancedManifest, result: &mut ValidationResult) {
        if manifest.project.name.is_empty() {
            result.add_warning(
                "project.name",
                "RECOMMENDED: Add 'project.name' for better error messages and multi-project support"
            );
        }

        // 0.0.0 is our 'default' for missing version, so warn if it looks like that
        if manifest.project.version == semver::Version::new(0, 0, 0) {
            result.add_warning(
                "project.version",
                "RECOMMENDED: Add 'project.version' to track environment changes over time",
            );
        }

        if let Some(lockfile) = &manifest.lockfile {
            if !lockfile.generate {
                result.add_warning(
                    "lockfile.generate",
                    "RECOMMENDED: Enable lockfile generation for reproducible builds",
                );
            }
        }

        for (name, profile) in &manifest.profiles {
            if profile.description.is_empty() {
                result.add_warning(
                    format!("profiles.{}.description", name),
                    format!("RECOMMENDED: Add description for profile '{}'", name),
                );
            }
        }
    }

    fn validate_platform_constraints(manifest: &EnhancedManifest, result: &mut ValidationResult) {
        if let Some(platform) = &manifest.platform {
            for (platform_key, version_req) in &platform.requirements {
                if let Err(e) = VersionReq::parse(version_req) {
                    result.add_error(
                        format!("platform.requirements.{:?}", platform_key),
                        format!("Invalid version requirement '{}': {}", version_req, e),
                    );
                }
            }
        }
    }

    fn validate_dependencies(_manifest: &EnhancedManifest, _result: &mut ValidationResult) {
        // Dependencies are strict types now.
    }

    fn validate_profiles(manifest: &EnhancedManifest, result: &mut ValidationResult) {
        for (name, profile) in &manifest.profiles {
            for dep_group in &profile.dependencies {
                let exists = match dep_group.as_str() {
                    "dependencies" => !manifest.dependencies.is_empty(),
                    "dev-dependencies" => !manifest.dev_dependencies.is_empty(),
                    "test-dependencies" => !manifest.test_dependencies.is_empty(),
                    "build-dependencies" => !manifest.build_dependencies.is_empty(),
                    _ => manifest.group.contains_key(dep_group),
                };

                if !exists {
                    result.add_error(
                        format!("profiles.{}.dependencies", name),
                        format!(
                            "Profile '{}' references non-existent dependency group '{}'",
                            name, dep_group
                        ),
                    );
                }
            }

            for excluded in &profile.exclude_groups {
                if !manifest.group.contains_key(excluded) {
                    result.add_warning(
                        format!("profiles.{}.exclude_groups", name),
                        format!(
                            "Profile '{}' tries to exclude non-existent group '{}'",
                            name, excluded
                        ),
                    );
                }
            }
        }
    }

    fn validate_groups(manifest: &EnhancedManifest, result: &mut ValidationResult) {
        for (name, group) in &manifest.group {
            if group.dependencies.is_empty() {
                result.add_error(
                    format!("group.{}.dependencies", name),
                    format!("Dependency group '{}' has no dependencies", name),
                );
            }
        }
    }

    pub fn check_platform_compatibility(manifest: &EnhancedManifest) -> Result<()> {
        if let Some(platform) = &manifest.platform {
            let current_os = std::env::consts::OS;
            let current_arch = std::env::consts::ARCH;

            let os_enum: OperatingSystem =
                serde_json::from_value(serde_json::Value::String(current_os.to_string()))
                    .unwrap_or(OperatingSystem::Any);

            let arch_enum: CpuArchitecture =
                serde_json::from_value(serde_json::Value::String(current_arch.to_string()))
                    .unwrap_or(CpuArchitecture::Any);

            if !platform.platforms.contains(&OperatingSystem::Any)
                && !platform.platforms.contains(&os_enum)
            {
                let valid_list: Vec<String> = platform
                    .platforms
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect();
                anyhow::bail!(
                    "Platform '{}' is not supported. Supported: {}",
                    current_os,
                    valid_list.join(", ")
                );
            }

            if !platform.architectures.contains(&CpuArchitecture::Any)
                && !platform.architectures.contains(&arch_enum)
            {
                let valid_list: Vec<String> = platform
                    .architectures
                    .iter()
                    .map(|a| format!("{:?}", a))
                    .collect();
                anyhow::bail!(
                    "Architecture '{}' is not supported. Supported: {}",
                    current_arch,
                    valid_list.join(", ")
                );
            }
        }

        Ok(())
    }
}
