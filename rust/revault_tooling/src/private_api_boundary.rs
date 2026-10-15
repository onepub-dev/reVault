//! Conservative guard for two explicitly private experiment subtrees.
//! This is not a general Rust reachability checker: public surfaces elsewhere
//! remain subject to the original conservative inventory. Macro expansion and
//! arbitrary cfg evaluation remain outside this bounded source guard; changes
//! introducing generated public surfaces require explicit API review.
use crate::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use syn::visit::Visit;

const BOUNDARIES: [&str; 2] = [
    "file_format::candidate_files",
    "file_format::recovery_commitment",
];

pub(super) fn excluded(relative: &Path) -> bool {
    let path = relative.to_string_lossy().replace('\\', "/");
    path == "file_format/recovery_commitment.rs"
        || path == "file_format/candidate_files.rs"
        || path.starts_with("file_format/candidate_files/")
        || path.starts_with("file_format/recovery_commitment/")
}

#[derive(Default)]
struct Guard {
    aliases: BTreeMap<String, Vec<(String, Vec<String>)>>,
    surfaces: Vec<(String, Vec<String>)>,
    boundaries: BTreeSet<String>,
    globs: Vec<(String, Vec<String>)>,
}

fn is_public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

#[derive(Default)]
struct Paths(Vec<Vec<String>>);
impl<'ast> Visit<'ast> for Paths {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.0
            .push(path.segments.iter().map(|s| s.ident.to_string()).collect());
        syn::visit::visit_path(self, path);
    }
}

fn uses(tree: &syn::UseTree, prefix: Vec<String>, out: &mut Vec<(String, Vec<String>)>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut next = prefix;
            next.push(path.ident.to_string());
            uses(&path.tree, next, out);
        }
        syn::UseTree::Name(name) => {
            let mut path = prefix;
            let name = name.ident.to_string();
            if name != "self" {
                path.push(name.clone());
            }
            let alias = path.last().cloned().unwrap_or(name);
            out.push((alias, path));
        }
        syn::UseTree::Rename(rename) => {
            let mut path = prefix;
            if rename.ident != "self" {
                path.push(rename.ident.to_string());
            }
            out.push((rename.rename.to_string(), path));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                uses(item, prefix.clone(), out);
            }
        }
        syn::UseTree::Glob(_) => out.push(("*".into(), prefix)),
    }
}

impl Guard {
    fn paths(&mut self, module: &str, paths: Paths) {
        self.surfaces
            .extend(paths.0.into_iter().map(|p| (module.to_owned(), p)));
    }

    fn items(&mut self, module: &str, items: &[syn::Item]) -> Result {
        for item in items {
            match item {
                syn::Item::Mod(item) => {
                    for attribute in &item.attrs {
                        if attribute.path().is_ident("path") {
                            if let syn::Meta::NameValue(value) = &attribute.meta {
                                if let syn::Expr::Lit(literal) = &value.value {
                                    if let syn::Lit::Str(path) = &literal.lit {
                                        if path.value().split('/').any(|part| {
                                            matches!(
                                                part,
                                                "candidate_files"
                                                    | "candidate_files.rs"
                                                    | "recovery_commitment"
                                                    | "recovery_commitment.rs"
                                            )
                                        }) {
                                            return Err("Binding inventory review required: #[path] redirects into an excluded subtree".into());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let name = join(module, &item.ident.to_string());
                    if BOUNDARIES.contains(&name.as_str()) {
                        if is_public(&item.vis) {
                            return Err(format!(
                                "Binding inventory review required: {name} became public"
                            )
                            .into());
                        }
                        self.boundaries.insert(name);
                        continue;
                    }
                    // Test-only module internals are not public API. Boundary
                    // declarations above are still checked even when test-only.
                    if item.attrs.iter().any(|a| {
                        a.path().is_ident("cfg")
                            && a.parse_args::<syn::Path>()
                                .is_ok_and(|p| p.is_ident("test"))
                    }) {
                        continue;
                    }
                    if let Some((_, items)) = &item.content {
                        self.items(&name, items)?;
                    }
                }
                syn::Item::Use(item) => {
                    let mut imports = Vec::new();
                    uses(&item.tree, Vec::new(), &mut imports);
                    for (alias, path) in imports {
                        if alias == "*" {
                            self.globs.push((module.to_owned(), path.clone()));
                        } else {
                            self.aliases
                                .entry(join(module, &alias))
                                .or_default()
                                .push((module.to_owned(), path.clone()));
                        }
                        if is_public(&item.vis) {
                            self.surfaces.push((module.to_owned(), path));
                        }
                    }
                }
                syn::Item::Type(item) => {
                    let mut paths = Paths::default();
                    paths.visit_type(&item.ty);
                    // Private aliases can subsequently escape in a public signature.
                    self.aliases
                        .entry(join(module, &item.ident.to_string()))
                        .or_default()
                        .extend(paths.0.iter().cloned().map(|p| (module.to_owned(), p)));
                    if is_public(&item.vis) {
                        self.paths(module, paths);
                    }
                }
                syn::Item::Fn(item) if is_public(&item.vis) => {
                    let mut paths = Paths::default();
                    paths.visit_signature(&item.sig);
                    self.paths(module, paths);
                }
                syn::Item::Struct(item) => {
                    let mut paths = Paths::default();
                    if is_public(&item.vis) {
                        paths.visit_generics(&item.generics);
                    }
                    for field in &item.fields {
                        if is_public(&field.vis) {
                            paths.visit_type(&field.ty);
                        }
                    }
                    self.paths(module, paths);
                }
                syn::Item::Enum(item) if is_public(&item.vis) => {
                    let mut paths = Paths::default();
                    paths.visit_item_enum(item);
                    self.paths(module, paths);
                }
                syn::Item::Trait(item) if is_public(&item.vis) => {
                    let mut paths = Paths::default();
                    paths.visit_item_trait(item);
                    self.paths(module, paths);
                }
                syn::Item::Impl(item) => {
                    let mut paths = Paths::default();
                    // An impl on an excluded type outside its subtree requires review,
                    // even if the type itself is named through a private alias.
                    paths.visit_type(&item.self_ty);
                    for member in &item.items {
                        match member {
                            syn::ImplItem::Fn(method) if is_public(&method.vis) => {
                                paths.visit_signature(&method.sig)
                            }
                            syn::ImplItem::Type(ty)
                                if is_public(&ty.vis) || item.trait_.is_some() =>
                            {
                                paths.visit_type(&ty.ty)
                            }
                            _ => {}
                        }
                    }
                    self.paths(module, paths);
                }
                syn::Item::Const(item) if is_public(&item.vis) => {
                    let mut p = Paths::default();
                    p.visit_type(&item.ty);
                    self.paths(module, p);
                }
                syn::Item::Static(item) if is_public(&item.vis) => {
                    let mut p = Paths::default();
                    p.visit_type(&item.ty);
                    self.paths(module, p);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn resolves(
        &self,
        module: &str,
        path: &[String],
        seen: &mut BTreeSet<String>,
        include_ancestors: bool,
    ) -> Result<bool> {
        if path.is_empty() {
            return Ok(false);
        }
        let mut context: Vec<&str> = module.split("::").filter(|p| !p.is_empty()).collect();
        let mut index = 0;
        match path[0].as_str() {
            "crate" => {
                context.clear();
                index = 1;
            }
            "self" => {
                index = 1;
            }
            "super" => {
                while path.get(index).is_some_and(|p| p == "super") {
                    context.pop();
                    index += 1;
                }
            }
            _ => {}
        }
        let remainder = &path[index..];
        let qualified = join(&context.join("::"), &remainder.join("::"));
        if BOUNDARIES.iter().any(|b| {
            qualified == *b
                || qualified.starts_with(&format!("{b}::"))
                || (include_ancestors
                    && (qualified.is_empty() || b.starts_with(&format!("{qualified}::"))))
        }) {
            return Ok(true);
        }
        // Unqualified root module paths are valid in root reexports. Relative
        // resolution otherwise uses the actual module and explicit imports.
        for cut in (1..=remainder.len()).rev() {
            let key = join(&context.join("::"), &remainder[..cut].join("::"));
            if let Some(aliases) = self.aliases.get(&key) {
                if !seen.insert(key.clone()) {
                    return Err(
                        format!("Binding inventory review required: alias cycle at {key}").into(),
                    );
                }
                for (scope, target) in aliases {
                    let mut expanded = target.clone();
                    expanded.extend_from_slice(&remainder[cut..]);
                    if self.resolves(scope, &expanded, seen, include_ancestors)? {
                        return Ok(true);
                    }
                }
                seen.remove(&key);
                return Ok(false);
            }
        }
        Ok(false)
    }

    fn verify(&self) -> Result {
        for boundary in BOUNDARIES {
            if !self.boundaries.contains(boundary) {
                return Err(format!(
                    "Binding inventory review required: missing private boundary {boundary}"
                )
                .into());
            }
        }
        for (module, path) in &self.surfaces {
            if self.resolves(module, path, &mut BTreeSet::new(), false)? {
                return Err(format!(
                    "Binding inventory review required: public surface {module} uses {}",
                    path.join("::")
                )
                .into());
            }
        }
        for (module, path) in &self.globs {
            // A glob from an ancestor can import either excluded module by name;
            // a glob into either subtree can import its types through unknown names.
            if self.resolves(module, path, &mut BTreeSet::new(), true)?
                || path.last().is_some_and(|p| p == "file_format")
            {
                return Err(format!(
                    "Binding inventory review required: ambiguous glob in {module}: {}",
                    path.join("::")
                )
                .into());
            }
        }
        Ok(())
    }
}

fn join(module: &str, name: &str) -> String {
    if module.is_empty() {
        name.to_owned()
    } else if name.is_empty() {
        module.to_owned()
    } else {
        format!("{module}::{name}")
    }
}

pub(super) fn verify(root: &Path) -> Result {
    let mut guard = Guard::default();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|v| v.to_str()) != Some("rs")
        {
            continue;
        }
        let relative = entry.path().strip_prefix(root)?;
        if excluded(relative)
            || relative
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.contains("test"))
        {
            continue;
        }
        let mut parts: Vec<String> = relative
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let leaf = parts.pop().unwrap();
        if leaf != "mod.rs" && leaf != "lib.rs" {
            parts.push(leaf.trim_end_matches(".rs").to_owned());
        }
        let parsed = syn::parse_file(&std::fs::read_to_string(entry.path())?)?;
        guard.items(&parts.join("::"), &parsed.items)?;
    }
    guard.verify()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn review(extra: &str) -> Result {
        let source = format!("mod file_format {{ pub(crate) mod candidate_files {{ pub fn hidden() {{}} }} pub(crate) mod recovery_commitment {{}} pub(crate) mod current_header {{ pub fn probe() {{}} }} }} {extra}");
        let mut guard = Guard::default();
        guard.items("", &syn::parse_file(&source)?.items)?;
        guard.verify()
    }
    #[test]
    fn known_private_boundaries_and_unrelated_exports_are_allowed() {
        review("pub use file_format::current_header::probe; pub fn hidden() {} pub struct PageInspection; pub use PageInspection as Inspection;").unwrap();
        assert!(excluded(Path::new("file_format/candidate_files/tree.rs")));
        assert!(!excluded(Path::new("file_format/current_header.rs")));
        assert!(!excluded(Path::new("model/candidate_files.rs")));
    }
    #[test]
    fn direct_grouped_and_chained_alias_exposure_requires_review() {
        for source in [
            "pub use crate::file_format::candidate_files::Value;",
            "use crate::file_format::{candidate_files as hidden}; pub use hidden::Value;",
            "use crate::file_format::candidate_files::Value as A; type B = A; pub fn value() -> B { todo!() }",
            "mod nested { use super::file_format::candidate_files as hidden; pub use self::hidden::Value; }",
            "use crate::file_format::candidate_files::Value; impl Value { pub fn method() {} }",
        ] { assert!(review(source).is_err(), "{source}"); }
    }
    #[test]
    fn public_types_fields_and_aliases_cannot_expose_excluded_types() {
        for source in [
            "pub type Value = crate::file_format::recovery_commitment::Root;",
            "pub struct Holder { pub value: crate::file_format::candidate_files::Value }",
            "pub fn value() -> crate::file_format::candidate_files::Value { todo!() }",
        ] {
            assert!(review(source).is_err(), "{source}");
        }
    }
    #[test]
    fn glob_and_alias_cycles_fail_closed() {
        assert!(review("use crate::file_format::*;").is_err());
        assert!(review("use crate::file_format as f; use f::*;").is_err());
        assert!(
            review("#[path = \"file_format/candidate_files/mod.rs\"] pub mod exposed;").is_err()
        );
        assert!(review("use crate::file_format::candidate_files::*;").is_err());
        assert!(review("use A as B; use B as A; pub fn value() -> A { todo!() }").is_err());
    }
}
