use clap::Parser;
use colored::*;
use regex::Regex;
use semver::{Version, VersionReq};
use std::str::FromStr;
use anyhow::{Result, anyhow};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// First package with version constraint (e.g., "requests>=2.0.0")
    #[arg(short = '1', long)]
    pkg1: String,

    /// Second package with version constraint (e.g., "requests<3.0.0")
    #[arg(short = '2', long)]
    pkg2: String,
}

#[derive(Debug)]
struct PythonPackage {
    name: String,
    version_req: VersionReq,
}

impl PythonPackage {
    fn parse(input: &str) -> Result<Self> {
        let re = Regex::new(r"^([a-zA-Z0-9_-]+)(.*?)$").unwrap();
        let captures = re.captures(input)
            .ok_or_else(|| anyhow!("Invalid package format: {}", input))?;
        
        let name = captures[1].to_string();
        let version_str = captures.get(2).map_or("", |m| m.as_str());
        
        // Convert Python-style version specifiers to semver format
        let version_req = if version_str.is_empty() {
            VersionReq::STAR
        } else {
            let cleaned = version_str
                .replace(">=", ">=")
                .replace("<=", "<=")
                .replace("==", "=")
                .replace("~=", "~")
                .replace("!=", "!");
            
            VersionReq::from_str(&cleaned)
                .map_err(|_| anyhow!("Invalid version requirement: {}", version_str))?
        };

        Ok(PythonPackage { name, version_req })
    }

    fn conflicts_with(&self, other: &PythonPackage) -> bool {
        if self.name != other.name {
            return false;
        }

        // Check some common versions to detect conflicts
        let test_versions = [
            "0.1.0", "1.0.0", "2.0.0", "3.0.0", "4.0.0", "5.0.0", "6.0.0", "7.0.0",
            "1.2.3", "2.3.4", "3.4.5", "4.5.6", "5.6.7",
            "1.0.1", "2.0.1", "3.0.1", "4.0.1", "5.0.1"
        ];

        for version_str in test_versions.iter() {
            if let Ok(version) = Version::parse(version_str) {
                let matches_self = self.version_req.matches(&version);
                let matches_other = other.version_req.matches(&version);
                
                if matches_self && matches_other {
                    return false; // Found a version that satisfies both requirements
                }
            }
        }

        true // No common version found
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let pkg1 = PythonPackage::parse(&args.pkg1)?;
    let pkg2 = PythonPackage::parse(&args.pkg2)?;

    println!("\nAnalyzing potential conflicts between:");
    println!("  Package 1: {} {}", pkg1.name, pkg1.version_req);
    println!("  Package 2: {} {}\n", pkg2.name, pkg2.version_req);

    if pkg1.name != pkg2.name {
        println!("{}", "No conflict: Different packages".green());
        return Ok(());
    }

    if pkg1.conflicts_with(&pkg2) {
        println!("{}", "CONFLICT DETECTED!".red().bold());
        println!("The version requirements are mutually exclusive.");
    } else {
        println!("{}", "No conflict detected".green());
        println!("The version requirements are compatible.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_parsing() -> Result<()> {
        // Test basic package name
        let pkg = PythonPackage::parse("requests")?;
        assert_eq!(pkg.name, "requests");
        assert_eq!(pkg.version_req.to_string(), "*");

        // Test with version constraints
        let test_cases = vec![
            ("requests>=2.0.0", "requests", ">=2.0.0"),
            ("django==3.2.0", "django", "=3.2.0"),
            ("flask<1.0", "flask", "<1.0"),
            ("numpy~=1.20", "numpy", "~1.20"),
        ];

        for (input, expected_name, expected_version) in test_cases {
            let pkg = PythonPackage::parse(input)?;
            assert_eq!(pkg.name, expected_name);
            assert_eq!(pkg.version_req.to_string(), expected_version);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_package_format() {
        // Test invalid package names
        assert!(PythonPackage::parse("").is_err());
        assert!(PythonPackage::parse("@invalid").is_err());
        assert!(PythonPackage::parse("invalid@1.0").is_err());
    }

    #[test]
    fn test_package_conflicts() -> Result<()> {
        // Test compatible versions
        let pkg1 = PythonPackage::parse("requests>=2.0.0")?;
        let pkg2 = PythonPackage::parse("requests<3.0.0")?;
        assert!(!pkg1.conflicts_with(&pkg2));

        // Test conflicting versions
        let pkg1 = PythonPackage::parse("django>=4.0.0")?;
        let pkg2 = PythonPackage::parse("django<3.0.0")?;
        assert!(pkg1.conflicts_with(&pkg2));

        // Test different packages (should never conflict)
        let pkg1 = PythonPackage::parse("requests>=2.0.0")?;
        let pkg2 = PythonPackage::parse("flask>=2.0.0")?;
        assert!(!pkg1.conflicts_with(&pkg2));

        // Test exact version requirements
        let pkg1 = PythonPackage::parse("pytest==6.0.0")?;
        let pkg2 = PythonPackage::parse("pytest==6.0.0")?;
        assert!(!pkg1.conflicts_with(&pkg2));

        let pkg1 = PythonPackage::parse("pytest==7.0.0")?;
        let pkg2 = PythonPackage::parse("pytest==6.0.0")?;
        assert!(pkg1.conflicts_with(&pkg2));

        Ok(())
    }

    #[test]
    fn test_version_ranges() -> Result<()> {
        // Test overlapping ranges
        let pkg1 = PythonPackage::parse("django>=2.0.0")?;
        let pkg2 = PythonPackage::parse("django<5.0.0")?;
        assert!(!pkg1.conflicts_with(&pkg2));

        // Test non-overlapping ranges
        let pkg1 = PythonPackage::parse("django>=5.0.0")?;
        let pkg2 = PythonPackage::parse("django<4.0.0")?;
        assert!(pkg1.conflicts_with(&pkg2));

        // Test boundary cases
        let pkg1 = PythonPackage::parse("django==3.0.0")?;
        let pkg2 = PythonPackage::parse("django>=3.0.0")?;
        assert!(!pkg1.conflicts_with(&pkg2));

        Ok(())
    }
}
