use super::{
    compression_file_profile_for_path_and_data, compression_file_type_for_path,
    compression_file_type_for_path_and_data, CompressionFileProfile, CompressionFileType,
};
use alloc::format;
use alloc::vec::Vec;
use std::path::Path;

#[test]
fn classifies_json_logs_as_json_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("logs/events.jsonl")),
        CompressionFileType::JsonText
    );
}

#[test]
fn classifies_rust_sources_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("src/main.rs")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_dictionary_named_bin_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("dict_dictionary.bin")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_service_file_as_config_text_even_with_dict_prefix() {
    assert_eq!(
        compression_file_type_for_path(Path::new("dict_systemd-logind.service")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_dockerfile_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Dockerfile")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_makefile_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Makefile")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_prefixed_gitignore_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("repo_.gitignore")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_prefixed_cargo_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("repo_Cargo.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_cargo_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Cargo.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_prefixed_yarn_lock_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("repo_yarn.lock")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_go_sum_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("go.sum")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_gemfile_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Gemfile")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_gemfile_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Gemfile.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_pipfile_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("Pipfile.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_composer_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("composer.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn profiles_prefixed_package_lock_as_dependency_json_lockfile() {
    let sample = br#"{
  "name": "app",
  "lockfileVersion": 3,
  "packages": {
    "": { "name": "app" }
  }
}"#;
    assert_eq!(
        compression_file_profile_for_path_and_data(
            Path::new("generated_package-lock.json"),
            sample
        ),
        CompressionFileProfile::DependencyJsonLockfile
    );
}

#[test]
fn profiles_prefixed_cargo_lock_as_cargo_lock() {
    let sample = br#"[[package]]
name = "ruzstd"
version = "0.1.0"
"#;
    assert_eq!(
        compression_file_profile_for_path_and_data(Path::new("repo_Cargo.lock"), sample),
        CompressionFileProfile::CargoLock
    );
}

#[test]
fn profiles_prefixed_composer_lock_as_composer_lock() {
    let sample = br#"{
  "packages": [
    {
      "name": "vendor/package",
      "require": {
        "php": ">=8.2"
      },
      "source": {
        "reference": "0000000000000000000000000000000000000001"
      },
      "version": "1.0.0"
    }
  ]
}"#;
    assert_eq!(
        compression_file_profile_for_path_and_data(Path::new("generated_composer.lock"), sample),
        CompressionFileProfile::ComposerLock
    );
}

#[test]
fn profiles_prefixed_poetry_lock_as_small_text_lockfile() {
    let sample = br#"[[package]]
name = "dep"
version = "1.0.0"
"#;
    assert_eq!(
        compression_file_profile_for_path_and_data(Path::new("generated_poetry.lock"), sample),
        CompressionFileProfile::SmallTextLockfile
    );
}

#[test]
fn profiles_prefixed_pubspec_lock_as_small_text_lockfile() {
    let sample = br#"sdks:
  dart: ">=3.0.0 <4.0.0"
packages:
  args:
    dependency: transitive
"#;
    assert_eq!(
        compression_file_profile_for_path_and_data(Path::new("generated_pubspec.lock"), sample),
        CompressionFileProfile::SmallTextLockfile
    );
}

#[test]
fn profiles_unknown_path_from_dependency_json_sample() {
    let mut sample = Vec::new();
    sample.extend_from_slice(b"{\n  \"_meta\": {},\n  \"default\": {\n");
    for idx in 0..256 {
        sample.extend_from_slice(format!("    \"dep-{idx:04}\": {{\n").as_bytes());
        sample.extend_from_slice(b"      \"hashes\": [\n");
        sample.extend_from_slice(
                b"        \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
            );
        sample.extend_from_slice(b"      ],\n");
        sample.extend_from_slice(b"      \"version\": \"==1.0.0\"\n");
        sample.extend_from_slice(b"    },\n");
    }
    sample.extend_from_slice(b"  },\n  \"develop\": {\n");
    for idx in 0..256 {
        sample.extend_from_slice(format!("    \"devdep-{idx:04}\": {{\n").as_bytes());
        sample.extend_from_slice(b"      \"hashes\": [\n");
        sample.extend_from_slice(
                b"        \"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n",
            );
        sample.extend_from_slice(b"      ],\n");
        sample.extend_from_slice(b"      \"version\": \"==2.0.0\"\n");
        sample.extend_from_slice(b"    },\n");
    }
    sample.extend_from_slice(b"  }\n}\n");
    assert_eq!(
        compression_file_profile_for_path_and_data(Path::new("fixture.data"), &sample),
        CompressionFileProfile::DependencyJsonLockfile
    );
}

#[test]
fn classifies_go_mod_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("go.mod")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_poetry_lock_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("poetry.lock")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_requirements_txt_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("requirements.txt")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_prefixed_npmrc_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("repo_.npmrc")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_proto_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("schema/service.proto")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_dic_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("lexicon/en_US.dic")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_crate_as_archive_like() {
    assert_eq!(
        compression_file_type_for_path(Path::new("target/package/demo.crate")),
        CompressionFileType::ArchiveLike
    );
}

#[test]
fn classifies_wasm_as_binary_like() {
    assert_eq!(
        compression_file_type_for_path(Path::new("pkg/module.wasm")),
        CompressionFileType::BinaryLike
    );
}

#[test]
fn classifies_compound_archive_extensions() {
    assert_eq!(
        compression_file_type_for_path(Path::new("archives/release.tar.gz")),
        CompressionFileType::ArchiveLike
    );
}

#[test]
fn classifies_prefixed_package_json_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("repo_package.json")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_tsconfig_json_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("tsconfig.json")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_markdown_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("docs/notes.md")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_pyproject_toml_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("pyproject.toml")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_html_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("web/index.html")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_build_bazel_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("BUILD.bazel")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_workspace_as_code_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("WORKSPACE")),
        CompressionFileType::CodeText
    );
}

#[test]
fn classifies_pubspec_lock_as_dictionary_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("pubspec.lock")),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn classifies_pubspec_yaml_as_config_text() {
    assert_eq!(
        compression_file_type_for_path(Path::new("pubspec.yaml")),
        CompressionFileType::ConfigText
    );
}

#[test]
fn classifies_additional_named_configs() {
    for path in [
        "deno.json",
        "devcontainer.json",
        "buf.yaml",
        "turbo.json",
        "nx.json",
        "taskfile.yml",
        "wrangler.toml",
    ] {
        assert_eq!(
            compression_file_type_for_path(Path::new(path)),
            CompressionFileType::ConfigText,
            "{path}"
        );
    }
}

#[test]
fn classifies_additional_extensions() {
    let config_paths = [
        "terraform/main.tf",
        "terraform/dev.tfvars",
        "ios/Project.xcconfig",
        "ios/project.pbxproj",
        "resources/strings.resx",
    ];
    for path in config_paths {
        assert_eq!(
            compression_file_type_for_path(Path::new(path)),
            CompressionFileType::ConfigText,
            "{path}"
        );
    }

    let json_paths = ["policy/config.hjson", "resources/App.xcstrings"];
    for path in json_paths {
        assert_eq!(
            compression_file_type_for_path(Path::new(path)),
            CompressionFileType::JsonText,
            "{path}"
        );
    }

    let code_paths = ["infra/main.bicep", "nix/shell.nix"];
    for path in code_paths {
        assert_eq!(
            compression_file_type_for_path(Path::new(path)),
            CompressionFileType::CodeText,
            "{path}"
        );
    }
}

#[test]
fn samples_json_before_falling_back_to_unknown() {
    let sample = br#"{"packages":[{"name":"alpha","version":"1.0.0"}]}"#;
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::JsonText
    );
}

#[test]
fn samples_code_before_falling_back_to_unknown() {
    let sample = b"fn main() {\n    let value = call();\n    println!(\"{value}\");\n}\n";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::CodeText
    );
}

#[test]
fn samples_config_before_falling_back_to_unknown() {
    let sample =
        b"[package]\nname = \"demo\"\nversion = \"0.1.0\"\n[dependencies]\nserde = \"1\"\n";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::ConfigText
    );
}

#[test]
fn samples_cargo_lock_before_falling_back_to_unknown() {
    let mut sample = Vec::new();
    for idx in 0..16 {
        sample.extend_from_slice(b"[[package]]\n");
        sample.extend_from_slice(format!("name = \"crate-{idx}\"\n").as_bytes());
        sample.extend_from_slice(b"version = \"1.0.0\"\n");
        sample.extend_from_slice(
            b"checksum = \"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\n",
        );
    }
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), &sample),
        CompressionFileType::DictionaryText
    );
}

#[test]
fn samples_archive_signature_before_falling_back_to_unknown() {
    let sample = b"PK\x03\x04\x14\x00\x00\x00";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::ArchiveLike
    );
}

#[test]
fn samples_binary_signature_before_falling_back_to_unknown() {
    let sample = b"\x7FELF\x02\x01\x01";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::BinaryLike
    );
}

#[test]
fn plain_text_sample_can_stay_unknown() {
    let sample = b"this is a plain text blob without strong config or code markers\njust words and spaces repeated for a sample\n";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("payload"), sample),
        CompressionFileType::Unknown
    );
}

#[test]
fn known_path_wins_over_sample_fallback() {
    let sample = b"\x7FELF\x02\x01\x01";
    assert_eq!(
        compression_file_type_for_path_and_data(Path::new("config.yaml"), sample),
        CompressionFileType::ConfigText
    );
}
