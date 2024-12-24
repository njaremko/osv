require_relative "lib/osv/version"

Gem::Specification.new do |spec|
  spec.name = "osv"
  spec.version = OSV::VERSION
  spec.authors = ["Nathan Jaremko"]
  spec.email = ["nathan@jaremko.ca"]

  spec.summary = "CSV parser for Ruby"
  spec.description = <<-EOF
OSV is a high-performance CSV parser for Ruby, implemented in Rust. It wraps BurntSushi's csv-rs crate to provide fast CSV parsing with support for both hash-based and array-based row formats.

Features include:
- Flexible input sources (file paths, gzipped files, IO objects, strings)
- Configurable parsing options (headers, separators, quote chars)
- Support for both hash and array output formats
- Whitespace trimming options
- Strict or flexible parsing modes
- Significantly faster than Ruby's standard CSV library
EOF
  spec.homepage = "https://github.com/njaremko/osv"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/njaremko/osv"
  spec.metadata["readme_uri"] = "https://github.com/njaremko/osv/blob/main/README.md"
  spec.metadata["changelog_uri"] = "https://github.com/njaremko/osv/blob/main/CHANGELOG.md"
  spec.metadata["documentation_uri"] = "https://www.rubydoc.info/gems/osv"
  spec.metadata["funding_uri"] = "https://github.com/sponsors/njaremko"

  spec.files =
    Dir[
      "lib/**/*.rb",
      "lib/**/*.rbi",
      "ext/**/*",
      "LICENSE",
      "README.md",
      "Cargo.toml",
      "Cargo.lock",
      "Gemfile",
      "Rakefile"
    ]
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/osv/extconf.rb"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys", "~> 0.9.39"

  # only needed when developing or packaging your gem
  spec.add_development_dependency "rake-compiler", "~> 1.2.0"
end
