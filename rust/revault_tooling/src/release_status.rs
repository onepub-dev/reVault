//! Registry presence is not proof that a particular CI candidate was published.
use crate::Result;
use serde_json::Value;
use std::{fs, path::Path, time::Duration};

pub fn report(root: &Path) -> Result {
    let dart = fs::read_to_string(root.join("bindings/dart/pubspec.yaml"))?;
    let version = dart
        .lines()
        .find_map(|s| s.strip_prefix("version:"))
        .ok_or("Missing binding version")?
        .trim();
    let cli = fs::read_to_string(root.join("rust/revault_cli/Cargo.toml"))?
        .parse::<toml_edit::DocumentMut>()?;
    let cli_version = cli["package"]["version"]
        .as_str()
        .ok_or("Missing CLI version")?;
    println!("Local release versions: CLI {cli_version}, bindings {version}");
    println!("Registry versions (presence does not verify candidate provenance):");
    let queries = [
        (
            "CLI / crates.io",
            "https://crates.io/api/v1/crates/revault_cli".to_owned(),
            "/crate/max_version",
        ),
        (
            "Rust / crates.io",
            "https://crates.io/api/v1/crates/revault-api".to_owned(),
            "/crate/max_version",
        ),
        (
            "npm",
            "https://registry.npmjs.org/@onepub-dev%2Frevault-api/latest".to_owned(),
            "/version",
        ),
        (
            "Python",
            "https://pypi.org/pypi/revault-api/json".to_owned(),
            "/info/version",
        ),
        (
            "Dart",
            "https://pub.dev/api/packages/revault_api".to_owned(),
            "/latest/version",
        ),
        (
            "Ruby",
            "https://rubygems.org/api/v1/gems/revault_api.json".to_owned(),
            "/version",
        ),
        (
            "NuGet",
            "https://api.nuget.org/v3-flatcontainer/revault.api/index.json".to_owned(),
            "/versions",
        ),
        (
            "Go Git tag",
            "https://api.github.com/repos/onepub-dev/revault-api/tags?per_page=1".to_owned(),
            "/0/name",
        ),
        (
            "Swift Git tag",
            "https://api.github.com/repos/onepub-dev/revault-swift/tags?per_page=1".to_owned(),
            "/0/name",
        ),
        (
            "PHP Git tag",
            "https://api.github.com/repos/onepub-dev/revault-php/tags?per_page=1".to_owned(),
            "/0/name",
        ),
        (
            "Maven Java",
            "https://repo1.maven.org/maven2/dev/onepub/revault-api/maven-metadata.xml".to_owned(),
            "xml",
        ),
        (
            "Maven Kotlin",
            "https://repo1.maven.org/maven2/dev/onepub/revault-api-kotlin/maven-metadata.xml"
                .to_owned(),
            "xml",
        ),
    ];
    std::thread::scope(|scope| {
        let handles: Vec<_> = queries
            .into_iter()
            .map(|(name, url, pointer)| {
                scope.spawn(move || {
                    let result = (|| -> Result<String> {
                        let config = ureq::Agent::config_builder()
                            .timeout_global(Some(Duration::from_secs(15)))
                            .build();
                        let mut response = ureq::Agent::new_with_config(config)
                            .get(&url)
                            .header("User-Agent", "revault-tool release status")
                            .call()?;
                        if pointer == "xml" {
                            let text = response.body_mut().read_to_string()?;
                            return text
                                .split_once("<release>")
                                .and_then(|(_, tail)| tail.split_once("</release>"))
                                .map(|(v, _)| v.to_owned())
                                .ok_or_else(|| "Missing Maven release version".into());
                        }
                        let value: Value = response.body_mut().read_json()?;
                        let value = value.pointer(pointer).ok_or("Missing registry version")?;
                        Ok(value
                            .as_str()
                            .or_else(|| value.as_array()?.last()?.as_str())
                            .ok_or("Invalid registry version")?
                            .to_owned())
                    })();
                    match result {
                        Ok(version) => format!("  {name}: {version}"),
                        Err(error) => format!("  {name}: unknown ({error})"),
                    }
                })
            })
            .collect();
        for handle in handles {
            println!(
                "{}",
                handle
                    .join()
                    .unwrap_or_else(|_| "Registry query failed".into())
            );
        }
    });
    println!("LuaRocks, Homebrew and platform carriers: inspect candidate publication jobs; registry status not verified.");
    Ok(())
}
