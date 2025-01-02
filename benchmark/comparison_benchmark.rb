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
  age = 0
  headers = %w[
    id
    name
    age
    email
    city
    country
    salary
    department
    hire_date
    manager_id
    performance_score
    project_count
    active
    notes
    last_login
    description
    skills
    address
  ]
  CSV.open("benchmark/test.csv", "w", write_headers: true, headers: headers) do |csv|
    rows.times do |i|
      row_age = rand(18..80)
      age += row_age
      csv << [
        i,
        "Person#{i}",
        row_age,
        "person#{i}@example.com",
        "City#{i}",
        "Country#{i}",
        rand(30_000..200_000),
        %w[Engineering Sales Marketing HR Finance].sample,
        "2020-#{rand(1..12)}-#{rand(1..28)}",
        rand(1..1000),
        rand(1..5).to_f,
        rand(1..10),
        [true, false].sample,
        "",
        "",
        # Large quoted text with commas and quotes
        "A very long description of person #{i}'s background, including multiple, comma-separated clauses. The person has \"special\" skills and experience in various fields.",
        # Array-like quoted text with commas
        "Ruby,Python,JavaScript,\"DevOps\",\"Cloud Architecture\"",
        # Address with embedded newlines and quotes
        "123 Main St.\nApt \"B\"\nSuite 100"
      ]
    end
  end

  file_string = File.read("benchmark/test.csv")

  Zlib::GzipWriter.open("benchmark/test.csv.gz") do |gz|
    CSV
      .new(gz, write_headers: true, headers: headers)
      .tap { |csv| CSV.parse(file_string, headers: true) { |row| csv << row } }
  end

  str = StringIO.new(file_string)
  [str, age]
end

TEST_FILES = %w[benchmark/test.csv benchmark/test.csv.gz].freeze

begin
  # Create test files
  test_data, age = generate_test_data

  # Create gzipped version

  puts "Benchmarking with #{`wc -l benchmark/test.csv`.to_i} lines of data\n\n"

  Benchmark.ips do |x|
    x.config(time: 30, warmup: 5)

    x.report("CSV - StringIO") do
      count = 0
      io = StringIO.new(test_data.string)
      CSV.new(io).each { |row| count += row[2].to_i }
      io.close
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("FastCSV - StringIO") do
      count = 0
      io = StringIO.new(test_data.string)
      FastCSV.raw_parse(io) { |row| count += row[2].to_i }

      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - StringIO") do
      count = 0
      io = StringIO.new(test_data.string)
      OSV.for_each(io, result_type: :array) { |row| count += row[2].to_i }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("CSV - Hash output") do
      count = 0
      File.open("benchmark/test.csv") { |f| CSV.new(f, headers: true).each { |row| count += row["age"].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - Hash output") do
      count = 0
      File.open("benchmark/test.csv") { |f| OSV.for_each(f) { |row| count += row["age"].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("CSV - Array output") do
      count = 0
      File.open("benchmark/test.csv") { |f| CSV.new(f).each { |row| count += row[2].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - Array output") do
      count = 0
      File.open("benchmark/test.csv") { |f| OSV.for_each(f, result_type: :array) { |row| count += row[2].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("FastCSV - Array output") do
      count = 0
      File.open("benchmark/test.csv") { |f| FastCSV.raw_parse(f) { |row| count += row[2].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - Direct Open Array output") do
      count = 0
      OSV.for_each("benchmark/test.csv", result_type: :array) { |row| count += row[2].to_i }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - Gzipped") do
      count = 0
      Zlib::GzipReader.open("benchmark/test.csv.gz") do |gz|
        OSV.for_each(gz, result_type: :array) { |row| count += row[2].to_i }
      end
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("OSV - Gzipped Direct") do
      count = 0
      OSV.for_each("benchmark/test.csv.gz", result_type: :array) { |row| count += row[2].to_i }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("FastCSV - Gzipped") do
      count = 0
      Zlib::GzipReader.open("benchmark/test.csv.gz") { |gz| FastCSV.raw_parse(gz) { |row| count += row[2].to_i } }
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.report("CSV - Gzipped") do
      count = 0
      Zlib::GzipReader.open("benchmark/test.csv.gz") do |gz|
        CSV.new(gz, headers: true).each { |row| count += row["age"].to_i }
      end
      raise "Age mismatch: #{age} != #{count}" if age != count
    end

    x.compare!
  end
ensure
  # Cleanup test files even if the script fails or is interrupted
  FileUtils.rm_f(TEST_FILES)
end
