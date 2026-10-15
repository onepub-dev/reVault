# Historical reader documentation isolation

Candidate `f3b69ee0bbc99117616b6a3e125111de319fb445` cannot be promoted: [its required documentation job](https://github.com/onepub-dev/reVault/actions/runs/38398193470/job/115299396492) failed because the five historical compatibility readers retain library names also used by current packages.

The workflow excludes those five packages from the combined output and documents each in a separate target directory. Existing version-3 reader and facade checks remain; warnings are still errors. All six changed commands pass locally with pinned Rust 1.88, locked offline dependencies and two build jobs. [Results and raw-log hashes](results.json) distinguish this focused validation from replacement candidate CI. No runtime code changed.

The preceding historical-reader hook audit also passed all nine decoder checks. Its original-source allowance flag for the 0.0.6 reader reflects the already reviewed, equivalent inline-format change in `mirrors.rs`; it is not an unexpected production change. The three compression files differ before and after the hook only by the pinned formatter output, and both copied Vault source trees remain identical to their originals.

The exact f3 candidate's fresh CLI build and three clean official-database advisory audits remain preserved outside this documentation fix. No resource measurements were run on that ineligible candidate.
