use domain::system::platform::PlatformDetector;
use domain::system::registry::{InstalledToolsRegistry, InstalledVersion, ToolManager};
use semver::Version;
use std::path::PathBuf;

#[test]
fn test_platform_detection() {
    let info = PlatformDetector::detect();

    println!("Platform: {}", info);
    assert_ne!(info.os_type, domain::system::platform::OsType::Unknown);
    assert_ne!(info.arch, domain::system::platform::Architecture::Unknown);
}

#[test]
fn test_tool_family_intelligence() {
    let mut registry = InstalledToolsRegistry::new();

    // Simulate: User has npm and bun installed
    registry.add_version(InstalledVersion {
        tool: "npm".to_string(),
        version: Version::new(10, 2, 3),
        location: PathBuf::from("/usr/local/bin/npm"),
        managed_by: ToolManager::System,
    });

    registry.add_version(InstalledVersion {
        tool: "bun".to_string(),
        version: Version::new(1, 0, 20),
        location: PathBuf::from("/usr/local/bin/bun"),
        managed_by: ToolManager::Homebrew,
    });

    // Test 1: Get all JS package managers installed
    let js_managers = registry.get_family_installed("js-package-manager");
    assert_eq!(js_managers.len(), 2);
    println!("✅ Found {} JS package managers", js_managers.len());

    // Test 2: Get preferred (should be bun, not npm)
    let preferred = registry.get_family_preferred("js-package-manager").unwrap();
    assert_eq!(preferred.tool, "bun");
    println!("✅ Preferred: {} (due to preference order)", preferred.tool);

    // Test 3: Get recommendations for yarn (not installed)
    let recommendations = registry.get_recommendations("yarn");
    assert!(recommendations.contains(&"npm".to_string()));
    assert!(recommendations.contains(&"bun".to_string()));
    println!("✅ Recommendations for yarn: {:?}", recommendations);

    println!("\n🎉 Tool Family Intelligence Demo Complete!");
    println!("   User has: npm@10.2.3, bun@1.0.20");
    println!("   Preferred: bun (fastest)");
    println!("   If yarn requested: Suggest using bun or npm instead");
}

#[test]
fn test_python_family() {
    let mut registry = InstalledToolsRegistry::new();

    // Simulate: User has python3 but not python
    registry.add_version(InstalledVersion {
        tool: "python3".to_string(),
        version: Version::new(3, 11, 5),
        location: PathBuf::from("/usr/bin/python3"),
        managed_by: ToolManager::System,
    });

    // Get preferred python (should be python3)
    let preferred = registry.get_family_preferred("python").unwrap();
    assert_eq!(preferred.tool, "python3");
    println!("✅ Python family: Using python3@{}", preferred.version);

    // Get recommendations for python (not installed)
    let recommendations = registry.get_recommendations("python");
    assert!(recommendations.contains(&"python3".to_string()));
    println!("✅ Recommendations for 'python': {:?}", recommendations);
}
