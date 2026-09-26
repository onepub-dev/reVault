use super::CompressionFileType;

fn classify_extension_exact(lower_ext: &str) -> Option<CompressionFileType> {
    let family = match lower_ext.as_bytes().first().copied()? {
        b'7' => match lower_ext {
            "7z" => CompressionFileType::ArchiveLike,
            _ => return None,
        },
        b'a' => match lower_ext {
            "a" | "adoc" | "aff" | "apk" | "app" | "ar" | "asm" | "astro" | "avsc" | "awk" => {
                if matches!(lower_ext, "a" | "apk" | "ar") {
                    CompressionFileType::ArchiveLike
                } else if lower_ext == "aff" {
                    CompressionFileType::DictionaryText
                } else if matches!(lower_ext, "adoc" | "avsc") {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::CodeText
                }
            }
            _ => return None,
        },
        b'b' => match lower_ext {
            "bash" | "bat" | "bazel" | "bicep" | "bzl" => CompressionFileType::CodeText,
            "bib" | "bin" | "bz2" => {
                if lower_ext == "bin" {
                    CompressionFileType::BinaryLike
                } else if lower_ext == "bz2" {
                    CompressionFileType::ArchiveLike
                } else {
                    CompressionFileType::ConfigText
                }
            }
            _ => return None,
        },
        b'c' => match lower_ext {
            "c" | "cab" | "cc" | "cfg" | "cjs" | "class" | "clj" | "cljs" | "cljc" | "cmd"
            | "conf" | "config" | "cpp" | "crate" | "cs" | "csproj" | "css" | "csv" | "csh"
            | "cxx" => match lower_ext {
                "cab" | "crate" => CompressionFileType::ArchiveLike,
                "cfg" | "conf" | "config" | "csproj" | "csv" => CompressionFileType::ConfigText,
                "class" => CompressionFileType::BinaryLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'd' => match lower_ext {
            "d" | "dart" | "dat" | "deb" | "desktop" | "dic" | "dict" | "dll" | "dmg"
            | "dockerfile" => match lower_ext {
                "dat" | "dll" => CompressionFileType::BinaryLike,
                "deb" | "dmg" => CompressionFileType::ArchiveLike,
                "desktop" => CompressionFileType::ConfigText,
                "dic" | "dict" => CompressionFileType::DictionaryText,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'e' => match lower_ext {
            "ear" | "edn" | "elm" | "env" | "eot" | "erl" | "exe" | "ex" | "exs" => match lower_ext
            {
                "ear" => CompressionFileType::ArchiveLike,
                "env" => CompressionFileType::ConfigText,
                "eot" | "exe" => CompressionFileType::BinaryLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'f' => match lower_ext {
            "fish" | "fs" | "fsi" | "fsproj" | "fsx" => {
                if lower_ext == "fsproj" {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::CodeText
                }
            }
            _ => return None,
        },
        b'g' => match lower_ext {
            "gem" | "geojson" | "go" | "gql" | "gradle" | "graphql" | "groovy" | "gvy" | "gz" => {
                match lower_ext {
                    "gem" | "gz" => CompressionFileType::ArchiveLike,
                    "geojson" => CompressionFileType::JsonText,
                    "go" | "gql" | "gradle" | "graphql" | "groovy" | "gvy" => {
                        CompressionFileType::CodeText
                    }
                    _ => return None,
                }
            }
            _ => return None,
        },
        b'h' => match lower_ext {
            "h" | "har" | "hbs" | "hcl" | "hh" | "hjson" | "hpp" | "hrl" | "hs" | "htm"
            | "html" | "hxx" => match lower_ext {
                "har" => CompressionFileType::JsonText,
                "hcl" => CompressionFileType::ConfigText,
                "hjson" => CompressionFileType::JsonText,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'i' => match lower_ext {
            "img" | "ini" | "ipa" | "ipynb" | "iso" => match lower_ext {
                "ini" => CompressionFileType::ConfigText,
                "ipynb" => CompressionFileType::JsonText,
                "img" => CompressionFileType::BinaryLike,
                _ => CompressionFileType::ArchiveLike,
            },
            _ => return None,
        },
        b'j' => match lower_ext {
            "jar" | "java" | "jl" | "js" | "json" | "json5" | "jsonc" | "jsonl" | "jsx" => {
                match lower_ext {
                    "jar" => CompressionFileType::ArchiveLike,
                    "json" | "json5" | "jsonc" | "jsonl" => CompressionFileType::JsonText,
                    _ => CompressionFileType::CodeText,
                }
            }
            _ => return None,
        },
        b'k' => match lower_ext {
            "kdl" | "kt" | "kts" => {
                if lower_ext == "kdl" {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::CodeText
                }
            }
            _ => return None,
        },
        b'l' => match lower_ext {
            "less" | "lhs" | "lib" | "liquid" | "list" | "log" | "lua" | "lz4" | "lzma" => {
                match lower_ext {
                    "lib" => CompressionFileType::BinaryLike,
                    "list" | "log" => CompressionFileType::ConfigText,
                    "lz4" | "lzma" => CompressionFileType::ArchiveLike,
                    _ => CompressionFileType::CodeText,
                }
            }
            _ => return None,
        },
        b'm' => match lower_ext {
            "m" | "markdown" | "mcmeta" | "md" | "mdx" | "meson" | "mjs" | "ml" | "mli" | "mm"
            | "mount" | "msi" | "msix" => match lower_ext {
                "markdown" | "md" => CompressionFileType::ConfigText,
                "mcmeta" => CompressionFileType::JsonText,
                "mount" => CompressionFileType::ConfigText,
                "msi" | "msix" => CompressionFileType::ArchiveLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'n' => match lower_ext {
            "ndjson" | "nim" | "nix" | "node" | "nupkg" => match lower_ext {
                "ndjson" => CompressionFileType::JsonText,
                "nim" | "nix" => CompressionFileType::CodeText,
                "node" => CompressionFileType::BinaryLike,
                "nupkg" => CompressionFileType::ArchiveLike,
                _ => return None,
            },
            _ => return None,
        },
        b'o' => match lower_ext {
            "o" | "obj" | "objcopy" | "otf" => CompressionFileType::BinaryLike,
            _ => return None,
        },
        b'p' => match lower_ext {
            "pak" | "path" | "pbxproj" | "pdf" | "php" | "phpt" | "pkg" | "pl" | "plist" | "pm"
            | "policy" | "properties" | "props" | "proto" | "ps1" | "psv" | "pyd" | "py"
            | "pyc" | "pyi" | "pyw" => match lower_ext {
                "pak" | "pkg" => CompressionFileType::ArchiveLike,
                "path" | "pbxproj" | "plist" | "policy" | "properties" | "props" | "psv" => {
                    CompressionFileType::ConfigText
                }
                "pdf" | "pyd" | "pyc" => CompressionFileType::BinaryLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b'r' => match lower_ext {
            "r" | "rar" | "rb" | "repo" | "resx" | "rlib" | "ron" | "rpm" | "rs" | "rst"
            | "rules" | "rkt" => match lower_ext {
                "rar" | "rpm" => CompressionFileType::ArchiveLike,
                "repo" | "resx" | "ron" | "rst" | "rules" => CompressionFileType::ConfigText,
                "rlib" => CompressionFileType::ArchiveLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b's' => match lower_ext {
            "s" | "sass" | "scala" | "scss" | "sed" | "service" | "sh" | "sln" | "so"
            | "socket" | "sql" | "svg" | "svelte" | "swift" => match lower_ext {
                "service" | "sln" | "socket" | "svg" => CompressionFileType::ConfigText,
                "so" => CompressionFileType::BinaryLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        b't' => match lower_ext {
            "tar" | "target" | "tcl" | "tex" | "tf" | "tfvars" | "tgz" | "thrift" | "timer"
            | "toml" | "topojson" | "ts" | "tsv" | "tsx" | "ttf" | "twig" | "txz" | "tzst" => {
                match lower_ext {
                    "tar" | "tgz" | "txz" | "tzst" => CompressionFileType::ArchiveLike,
                    "target" | "tex" | "tf" | "tfvars" | "timer" | "toml" | "tsv" => {
                        CompressionFileType::ConfigText
                    }
                    "topojson" => CompressionFileType::JsonText,
                    "ttf" => CompressionFileType::BinaryLike,
                    _ => CompressionFileType::CodeText,
                }
            }
            _ => return None,
        },
        b'v' => match lower_ext {
            "vbproj" | "vcxproj" | "vue" => {
                if lower_ext == "vue" {
                    CompressionFileType::CodeText
                } else {
                    CompressionFileType::ConfigText
                }
            }
            _ => return None,
        },
        b'w' => match lower_ext {
            "war" | "wasm" | "webmanifest" | "whl" | "woff" | "woff2" => match lower_ext {
                "war" | "whl" => CompressionFileType::ArchiveLike,
                "webmanifest" => CompressionFileType::JsonText,
                _ => CompressionFileType::BinaryLike,
            },
            _ => return None,
        },
        b'x' => match lower_ext {
            "xaml" | "xapk" | "xconfig" | "xhtml" | "xcconfig" | "xcstrings" | "xml" | "xpi"
            | "xsd" | "xsl" | "xslt" | "xz" => match lower_ext {
                "xapk" | "xpi" | "xz" => CompressionFileType::ArchiveLike,
                "xaml" | "xconfig" | "xcconfig" | "xml" | "xsd" | "xsl" | "xslt" => {
                    CompressionFileType::ConfigText
                }
                "xcstrings" => CompressionFileType::JsonText,
                "xhtml" => CompressionFileType::CodeText,
                _ => return None,
            },
            _ => return None,
        },
        b'y' => match lower_ext {
            "yaml" | "yml" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'z' => match lower_ext {
            "zig" | "zip" | "zsh" | "zst" => match lower_ext {
                "zip" | "zst" => CompressionFileType::ArchiveLike,
                _ => CompressionFileType::CodeText,
            },
            _ => return None,
        },
        _ => return None,
    };
    Some(family)
}

fn classify_named_file_exact(lower_name: &str) -> Option<CompressionFileType> {
    let family = match lower_name.as_bytes().first().copied()? {
        b'.' => match lower_name {
            ".babelrc" | ".clang-format" | ".clang-tidy" | ".coveragerc" | ".dockerignore"
            | ".editorconfig" | ".env" | ".env.development" | ".env.local" | ".env.production"
            | ".env.test" | ".eslintignore" | ".eslintrc" | ".flake8" | ".gitattributes"
            | ".gitignore" | ".gitmodules" | ".mailmap" | ".npmignore" | ".npmrc"
            | ".prettierignore" | ".prettierrc" | ".pylintrc" | ".shellcheckrc"
            | ".stylelintrc" | ".yamllint" | ".yarnrc" | ".yarnrc.yml" => {
                CompressionFileType::ConfigText
            }
            _ => return None,
        },
        b'a' => match lower_name {
            "api-extractor.json" | "appveyor.yml" | "azure-pipelines.yml" => {
                CompressionFileType::ConfigText
            }
            _ => return None,
        },
        b'b' => match lower_name {
            "brewfile" => CompressionFileType::ConfigText,
            "bsdmakefile" | "build.bazel" | "buildfile" => CompressionFileType::CodeText,
            "build.gradle" | "build.gradle.kts" => CompressionFileType::ConfigText,
            "buf.gen.yaml" | "buf.work.yaml" | "buf.yaml" => CompressionFileType::ConfigText,
            "bun.lock" => CompressionFileType::DictionaryText,
            _ => return None,
        },
        b'c' => match lower_name {
            "cargo.lock" => CompressionFileType::DictionaryText,
            "cmakelists.txt" => CompressionFileType::CodeText,
            "composer.json" => CompressionFileType::ConfigText,
            "composer.lock" | "containerfile" => {
                if lower_name == "containerfile" {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::DictionaryText
                }
            }
            _ => return None,
        },
        b'd' => match lower_name {
            "deno.json" | "deno.jsonc" | "devcontainer.json" => CompressionFileType::ConfigText,
            "docker-compose.yaml" | "docker-compose.yml" | "dockerfile" => {
                CompressionFileType::ConfigText
            }
            _ => return None,
        },
        b'g' => match lower_name {
            "gemfile" => CompressionFileType::ConfigText,
            "gemfile.lock" | "go.sum" => CompressionFileType::DictionaryText,
            "gnumakefile" | "go.mod" => {
                if lower_name == "go.mod" {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::CodeText
                }
            }
            "gradle.properties" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'j' => match lower_name {
            "jenkinsfile" | "justfile" => CompressionFileType::CodeText,
            "jsconfig.json" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'l' => match lower_name {
            "lerna.json" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'm' => match lower_name {
            "makefile" | "melos.yaml" | "meson.build" | "meson_options.txt" | "module.bazel" => {
                if lower_name == "melos.yaml" {
                    CompressionFileType::ConfigText
                } else {
                    CompressionFileType::CodeText
                }
            }
            "mix.lock" => CompressionFileType::DictionaryText,
            _ => return None,
        },
        b'n' => match lower_name {
            "netlify.toml" | "nx.json" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'p' => match lower_name {
            "package.json" => CompressionFileType::ConfigText,
            "pipfile"
            | "pnpm-workspace.yaml"
            | "podfile"
            | "pom.xml"
            | "procfile"
            | "pubspec.yaml"
            | "pyproject.toml" => CompressionFileType::ConfigText,
            "pipfile.lock" | "podfile.lock" | "pnpm-lock.yaml" | "pubspec.lock" => {
                CompressionFileType::DictionaryText
            }
            "poetry.lock" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'r' => match lower_name {
            "rakefile" => CompressionFileType::CodeText,
            "release-please-config.json" | "renovate.json" => CompressionFileType::ConfigText,
            "requirements.txt" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b's' => match lower_name {
            "settings.gradle" | "settings.gradle.kts" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b't' => match lower_name {
            "taskfile.yml" => CompressionFileType::ConfigText,
            "tiltfile" => CompressionFileType::CodeText,
            "tsconfig.json" | "turbo.json" | "typedoc.json" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'v' => match lower_name {
            "vagrantfile" => CompressionFileType::ConfigText,
            _ => return None,
        },
        b'w' => match lower_name {
            "wrangler.toml" => CompressionFileType::ConfigText,
            "workspace" => CompressionFileType::CodeText,
            _ => return None,
        },
        b'y' => match lower_name {
            "yarn.lock" => CompressionFileType::ConfigText,
            _ => return None,
        },
        _ => return None,
    };
    Some(family)
}

pub(super) fn classify_file_name(lower_name: &str) -> CompressionFileType {
    if lower_name.contains("dictionary") {
        return CompressionFileType::DictionaryText;
    }

    if let Some(family) = classify_named_file_exact(lower_name) {
        return family;
    }

    for suffix in file_name_prefixed_suffixes(lower_name) {
        if let Some(family) = classify_named_file_exact(suffix) {
            return family;
        }
    }

    for (idx, byte) in lower_name.bytes().enumerate() {
        if byte == b'.' && idx + 1 < lower_name.len() {
            if let Some(family) = classify_extension_exact(&lower_name[idx + 1..]) {
                return family;
            }
        }
    }

    CompressionFileType::Unknown
}

fn file_name_prefixed_suffixes(lower_name: &str) -> impl Iterator<Item = &str> {
    lower_name
        .bytes()
        .enumerate()
        .filter_map(move |(idx, byte)| {
            if matches!(byte, b'_' | b'-' | b'.') && idx + 1 < lower_name.len() {
                Some(&lower_name[idx + 1..])
            } else {
                None
            }
        })
}

pub(super) fn file_name_matches_exact_or_prefixed(lower_name: &str, needle: &str) -> bool {
    lower_name == needle || file_name_prefixed_suffixes(lower_name).any(|suffix| suffix == needle)
}
