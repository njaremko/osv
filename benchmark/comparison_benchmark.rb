#!/usr/bin/env ruby
# frozen_string_literal: true

require "benchmark/ips"
require "csv"
require "osv"
require "fastcsv"
require "stringio"
require "zlib"
require "fileutils"

RubyVM::YJIT.enable

# Generate a larger test file for more meaningful benchmarks
def generate_test_data(rows = 1_000_000)
  headers = %w[id name age email city country]
  StringIO.new.tap do |io|
    io.puts headers.join(",")
    rows.times do |i|
      row = [i, "Person#{i}", rand(18..80), "person#{i}@example.com", "City#{i}", "Country#{i}"]
      io.puts row.join(",")
    end
    io.rewind
  end
end

TEST_FILES = %w[benchmark/test.csv benchmark/test.csv.gz].freeze

begin
  # Create test files
  test_data = generate_test_data.string
  File.write("benchmark/test.csv", test_data)
  io = StringIO.new(test_data)

  # Create gzipped version
  Zlib::GzipWriter.open("benchmark/test.csv.gz") { |gz| gz.write(test_data) }

  puts "Benchmarking with #{`wc -l benchmark/test.csv`.to_i} lines of data\n\n"

  Benchmark.ips do |x|
    x.config(time: 10, warmup: 10)

    x.report("OSV - StringIO") do
      result = []
      OSV.for_each(io) { |row| result << row }
      io.rewind
      result
    end

    x.report("FastCSV - StringIO") do
      result = []
      FastCSV.raw_parse(io) { |row| result << row }
      io.rewind
      result
    end

    x.report("OSV - Hash output") do
      result = []
      File.open("benchmark/test.csv") { |f| OSV.for_each(f) { |row| result << row } }
      result
    end

    x.report("OSV - Array output") do
      result = []
      File.open("benchmark/test.csv") { |f| OSV.for_each(f, result_type: :array) { |row| result << row } }
      result
    end

    x.report("OSV - Direct Open Array output") do
      result = []
      OSV.for_each("benchmark/test.csv", result_type: :array) { |row| result << row }
      result
    end

    x.report("OSV - Gzipped") do
      result = []
      Zlib::GzipReader.open("benchmark/test.csv.gz") do |gz|
        OSV.for_each(gz, result_type: :array) { |row| result << row }
      end
      result
    end

    x.report("OSV - Gzipped Direct") do
      result = []
      OSV.for_each("benchmark/test.csv.gz", result_type: :array) { |row| result << row }
      result
    end

    x.report("FastCSV - Array output") do
      result = []
      File.open("benchmark/test.csv") { |f| FastCSV.raw_parse(f) { |row| result << row } }
      result
    end

    x.report("FastCSV - Gzipped") do
      result = []
      Zlib::GzipReader.open("benchmark/test.csv.gz") { |gz| FastCSV.raw_parse(gz) { |row| result << row } }
      result
    end

    # x.report("CSV - Gzipped") do
    #   Zlib::GzipReader.open("benchmark/test.csv.gz") { |gz| CSV.new(gz, headers: true).map(&:to_h) }
    # end

    # x.report("CSV - Hash output") { File.open("benchmark/test.csv") { |f| CSV.new(f, headers: true).map(&:to_h) } }

    # x.report("CSV - Array output") { File.open("benchmark/test.csv") { |f| CSV.new(f).read } }

    # x.report("CSV - StringIO") do
    #   result = []
    #   io = StringIO.new(test_data)
    #   CSV.new(io).each { |row| result << row }
    #   io.close

    #   result
    # end

    x.compare!
  end
ensure
  # Cleanup test files even if the script fails or is interrupted
  FileUtils.rm_f(TEST_FILES)
end
