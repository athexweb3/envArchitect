use domain::intelligence::MetricsDetector;
use domain::system::PlatformDetector;

#[test]
fn test_npm_package_sizes() {
    let platform = PlatformDetector::detect();
    let detector = MetricsDetector::new(platform);

    println!("\n📦 npm Package Sizes:");
    println!("====================");

    let packages = vec!["typescript", "webpack", "vite", "react", "express"];
    for pkg in packages {
        let size = detector.get_tool_size(pkg).unwrap();
        println!("  {}: {}MB", pkg, size);
    }
}

#[test]
fn test_pypi_package_sizes() {
    let platform = PlatformDetector::detect();
    let detector = MetricsDetector::new(platform);

    println!("\n🐍 PyPI Package Sizes:");
    println!("====================");

    let packages = vec!["django", "flask", "numpy", "pandas", "tensorflow"];
    for pkg in packages {
        let size = detector.get_tool_size(pkg).unwrap();
        println!("  {}: {}MB", pkg, size);
    }
}

#[test]
fn test_crates_io_sizes() {
    let platform = PlatformDetector::detect();
    let detector = MetricsDetector::new(platform);

    println!("\n🦀 crates.io Package Sizes:");
    println!("===========================");

    let crates = vec!["serde", "tokio", "actix-web", "rocket", "diesel"];
    for cr in crates {
        let size = detector.get_tool_size(cr).unwrap();
        println!("  {}: {}MB", cr, size);
    }
}

#[test]
fn test_fallback_to_os_packages() {
    let platform = PlatformDetector::detect();
    let detector = MetricsDetector::new(platform);

    println!("\n💻 OS Package Sizes (fallback):");
    println!("===============================");

    let tools = vec!["nodejs", "python3", "rust", "go"];
    for tool in tools {
        let size = detector.get_tool_size(tool).unwrap();
        println!("  {}: {}MB", tool, size);
    }
}
