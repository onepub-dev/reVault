Gem::Specification.new do |spec|
  spec.name = "revault_api"
  spec.version = "0.2.0"
  spec.summary = "Ruby Fiddle bindings for reVault"
  spec.description = "Complete class-based reVault lockbox and vault API"
  spec.authors = ["OnePub"]
  spec.email = ["bsutton@onepub.dev"]
  spec.homepage = "https://docs.revault.onepub.dev/"
  spec.license = "Nonstandard"
  spec.metadata = {
    "bug_tracker_uri" => "https://github.com/onepub-dev/reVault/issues",
    "changelog_uri" => "https://github.com/onepub-dev/reVault/blob/main/bindings/CHANGELOG.md",
    "documentation_uri" => "https://docs.revault.onepub.dev/",
    "source_code_uri" => "https://github.com/onepub-dev/reVault/tree/main/bindings/ruby"
  }
  spec.required_ruby_version = ">= 3.1"
  spec.platform = "@GEM_PLATFORM@"
  spec.files = Dir["revault_api.rb", "lib/**/*.rb", "generated/**/*.rb", "native/**/*", "README.md", "LICENSE"]
  spec.require_paths = ["."]
end
