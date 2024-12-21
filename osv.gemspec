require_relative "lib/osv/version"

Gem::Specification.new do |spec|
  spec.name = "osv"
  spec.version = OSV::VERSION
  spec.authors = ["Nathan Jaremko"]
  spec.email = ["nathan@jaremko.ca"]

  spec.summary = "CSV parser for Ruby"
  spec.homepage = "https://github.com/njaremko/osv"
  spec.license = "NONE"
  spec.required_ruby_version = ">= 3.1.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/njaremko/osv"

  spec.files = Dir["lib/**/*.rb", "LICENSE", "README.md", "Cargo.*", "Gemfile", "Rakefile", "lib/osv/osv.bundle"]
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/osv/extconf.rb"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys", "~> 0.9.39"

  # only needed when developing or packaging your gem
  spec.add_development_dependency "rake-compiler", "~> 1.2.0"
end
