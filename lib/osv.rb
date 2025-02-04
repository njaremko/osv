require_relative "osv/version"

begin
  require "osv/#{RUBY_VERSION.to_f}/osv"
rescue LoadError
  require "osv/osv"
end

module OSV
end
