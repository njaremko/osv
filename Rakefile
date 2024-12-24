# frozen_string_literal: true

require "rake/testtask"
require "rake/extensiontask"

task default: :test

Rake::ExtensionTask.new("osv") do |c|
  c.lib_dir = "lib/osv"
  c.ext_dir = "ext/osv"
end

task :dev do
  ENV["RB_SYS_CARGO_PROFILE"] = "release"
end

Rake::TestTask.new do |t|
  t.deps << :dev << :compile
  t.test_files = FileList[File.expand_path("test/*_test.rb", __dir__)]
  t.libs << "lib"
  t.libs << "test"
end
