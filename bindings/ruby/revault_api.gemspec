Gem::Specification.new do |spec|
  spec.name = "revault_api"
  spec.version = "0.2.0"
  spec.summary = "Ruby Fiddle bindings for reVault"
  spec.description = "Complete class-based reVault lockbox and vault API"
  spec.authors = ["OnePub"]
  spec.homepage = "https://github.com/onepub-dev/reVault"
  spec.license = "Nonstandard"
  spec.required_ruby_version = ">= 3.1"
  spec.platform = "@GEM_PLATFORM@"
  spec.files = Dir["revault_api.rb", "lib/**/*.rb", "generated/**/*.rb", "native/**/*", "LICENSE"]
  spec.require_paths = ["."]
  spec.add_runtime_dependency "google-protobuf", "~> 3.25"
end
