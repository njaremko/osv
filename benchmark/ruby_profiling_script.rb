#!/usr/bin/env ruby
# frozen_string_literal: true

require "osv"
require "fastcsv"
require "stringio"
require "time"

# Generate a larger test file for more meaningful benchmarks
def generate_test_data(rows = 1_000_000)
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
  ]
  StringIO.new.tap do |io|
    io.puts headers.join(",")
    rows.times do |i|
      row = [
        i,
        "Person#{i}",
        rand(18..80),
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
        ""
      ]
      io.puts row.join(",")
    end
    io.rewind
  end
end

# Generate test data and write to file
test_data = generate_test_data.string
File.write("benchmark_test.csv", test_data)

io = StringIO.new(test_data)

# Process the file in a loop for 10 seconds
end_time = Time.now + 10
iterations = 0

while Time.now < end_time
  count = 0
  OSV.for_each(io, result_type: :array) { |row| count += row[2].to_i }
  # FastCSV.raw_parse(io) { |row| count += row[2].to_i }
  io.rewind
  iterations += 1
end

puts "Completed #{iterations} iterations in 10 seconds"

# Cleanup
File.delete("benchmark_test.csv")
