# frozen_string_literal: true

require "rake/testtask"
require "rb_sys/extensiontask"

task default: :test

GEMSPEC = Gem::Specification.load("osv.gemspec")

RbSys::ExtensionTask.new("osv", GEMSPEC) do |ext|
  ext.lib_dir = "lib/osv"
  ext.ext_dir = "ext/osv"
end

Rake::TestTask.new do |t|
  t.deps << :compile
  t.test_files = FileList[File.expand_path("test/*_test.rb", __dir__)]
  t.libs << "lib"
  t.libs << "test"
end

task :release do
  sh "bundle exec rake test"
  sh "mkdir -p pkg"
  sh "gem build osv.gemspec -o pkg/osv-#{OSV::VERSION}.gem"
  sh "gem push pkg/osv-#{OSV::VERSION}.gem"
end
