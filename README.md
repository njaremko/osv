# OSV

[![Gem Version](https://badge.fury.io/rb/osv.svg)](https://badge.fury.io/rb/osv)

OSV is a high-performance CSV parser for Ruby, implemented in Rust. It wraps BurntSushi's excellent [csv-rs](https://github.com/BurntSushi/rust-csv) crate.

It provides a simple interface for reading CSV files with support for both hash-based and array-based row formats.

The array-based mode is faster than the hash-based mode, so if you don't need the hash keys, use the array-based mode.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'osv'
```

And then execute:

```bash
bundle install
```

Or install it directly:

```bash
gem install osv
```

## Usage

### Reading CSV Files

```ruby
require 'osv'

# Basic usage - each row as a hash
OSV.for_each("data.csv") do |row|
  puts row["name"]  # => "John"
  puts row["age"]   # => "25"
end

# Return an enumerator instead of using a block
rows = OSV.for_each("data.csv")
rows.each { |row| puts row["name"] }

# High-performance array mode
OSV.for_each("data.csv", result_type: :array) do |row|
  puts row[0]  # First column
  puts row[1]  # Second column
end
```

### Input Sources

```ruby
# From a file path
OSV.for_each("data.csv") { |row| puts row["name"] }

# From a file path
OSV.for_each("data.csv.gz") { |row| puts row["name"] }

# From an IO object
File.open("data.csv") { |file| OSV.for_each(file) { |row| puts row["name"] } }

# From a string
data = StringIO.new("name,age\nJohn,25")
OSV.for_each(data) { |row| puts row["name"] }
```

### Configuration Options

```ruby
OSV.for_each("data.csv",
  # Input formatting
  has_headers: true,     # First row contains headers (default: true)
  col_sep: ",",          # Column separator (default: ",")
  quote_char: '"',       # Quote character (default: '"')

  # Output formatting
  result_type: :hash,    # :hash or :array (hash is default)
  nil_string: nil,       # String to interpret as nil when parsing (default: nil)

  # Parsing behavior
  flexible: false,       # Allow varying number of fields (default: false)
  flexible_default: nil, # Default value for missing fields. If unset, we ignore missing fields.
                         # Implicitly enables flexible mode if set.
  trim: :all,            # Whether to trim whitespace. Options are :all, :headers, or :fields (default: nil)
  buffer_size: 1024,     # Number of rows to buffer in memory (default: 1024)
)
```

#### Available Options

- `has_headers`: Boolean indicating if the first row contains headers (default: true)
- `col_sep`: String specifying the field separator (default: ",")
- `quote_char`: String specifying the quote character (default: "\"")
- `nil_string`: String that should be interpreted as nil
  - by default, empty strings are interpreted as empty strings
  - if you want to interpret empty strings as nil, set this to an empty string
- `buffer_size`: Integer specifying the number of rows to buffer in memory (default: 1024)
- `result_type`: String specifying the output format ("hash" or "array" or :hash or :array)
- `flexible`: Boolean specifying if the parser should be flexible (default: false)
- `flexible_default`: String specifying the default value for missing fields. Implicitly enables flexible mode if set. (default: `nil`)
- `trim`: String specifying the trim mode ("all" or "headers" or "fields" or :all or :headers or :fields)

When `has_headers` is false, hash keys will be generated as `"c0"`, `"c1"`, etc.

## Requirements

- Ruby >= 3.1.0
- Rust toolchain (for installation from source)

## Performance

This library is faster than the standard Ruby CSV library. It's also faster than any other CSV gem I've been able to find.

Here's some unscientific benchmarks. You can find the code in the [benchmark/comparison_benchmark.rb](benchmark/comparison_benchmark.rb) file.

### 1,000,000 records

```
🏃  Running benchmarks...
Benchmarking with 3000001 lines of data

ruby 3.3.6 (2024-11-05 revision 75015d4c1f) +YJIT [arm64-darwin24]
Warming up --------------------------------------
      CSV - StringIO     1.000 i/100ms
  FastCSV - StringIO     1.000 i/100ms
      OSV - StringIO     1.000 i/100ms
   CSV - Hash output     1.000 i/100ms
   OSV - Hash output     1.000 i/100ms
  CSV - Array output     1.000 i/100ms
  OSV - Array output     1.000 i/100ms
FastCSV - Array output
                         1.000 i/100ms
OSV - Direct Open Array output
                         1.000 i/100ms
       OSV - Gzipped     1.000 i/100ms
OSV - Gzipped Direct     1.000 i/100ms
   FastCSV - Gzipped     1.000 i/100ms
       CSV - Gzipped     1.000 i/100ms
Calculating -------------------------------------
      CSV - StringIO      0.079 (± 0.0%) i/s    (12.69 s/i) -      3.000 in  38.139709s
  FastCSV - StringIO      0.370 (± 0.0%) i/s     (2.71 s/i) -     12.000 in  32.474164s
      OSV - StringIO      0.635 (± 0.0%) i/s     (1.58 s/i) -     19.000 in  30.772490s
   CSV - Hash output      0.058 (± 0.0%) i/s    (17.11 s/i) -      2.000 in  34.212335s
   OSV - Hash output      0.249 (± 0.0%) i/s     (4.01 s/i) -      8.000 in  32.124319s
  CSV - Array output      0.066 (± 0.0%) i/s    (15.11 s/i) -      2.000 in  30.212137s
  OSV - Array output      0.665 (± 0.0%) i/s     (1.50 s/i) -     20.000 in  30.813986s
FastCSV - Array output
                          0.351 (± 0.0%) i/s     (2.85 s/i) -     11.000 in  31.418786s
OSV - Direct Open Array output
                          0.713 (± 0.0%) i/s     (1.40 s/i) -     22.000 in  30.938525s
       OSV - Gzipped      0.506 (± 0.0%) i/s     (1.98 s/i) -     16.000 in  31.709708s
OSV - Gzipped Direct      0.685 (± 0.0%) i/s     (1.46 s/i) -     21.000 in  31.145435s
   FastCSV - Gzipped      0.324 (± 0.0%) i/s     (3.09 s/i) -     10.000 in  30.983582s
       CSV - Gzipped      0.057 (± 0.0%) i/s    (17.69 s/i) -      2.000 in  35.379009s

Comparison:
OSV - Direct Open Array output:        0.7 i/s
OSV - Gzipped Direct:        0.7 i/s - 1.04x  slower
  OSV - Array output:        0.7 i/s - 1.07x  slower
      OSV - StringIO:        0.6 i/s - 1.12x  slower
       OSV - Gzipped:        0.5 i/s - 1.41x  slower
  FastCSV - StringIO:        0.4 i/s - 1.93x  slower
FastCSV - Array output:        0.4 i/s - 2.03x  slower
   FastCSV - Gzipped:        0.3 i/s - 2.20x  slower
   OSV - Hash output:        0.2 i/s - 2.86x  slower
      CSV - StringIO:        0.1 i/s - 9.05x  slower
  CSV - Array output:        0.1 i/s - 10.77x  slower
   CSV - Hash output:        0.1 i/s - 12.20x  slower
       CSV - Gzipped:        0.1 i/s - 12.61x  slower
```
