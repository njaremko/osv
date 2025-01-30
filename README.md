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
  trim: :all,            # Whether to trim whitespace. Options are :all, :headers, or :fields (default: nil)
  buffer_size: 1024,     # Number of rows to buffer in memory (default: 1024)
  ignore_null_bytes: false, # Boolean specifying if null bytes should be ignored (default: false)
  lossy: false,             # Boolean specifying if invalid UTF-8 characters should be replaced with a replacement character (default: false)
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
- `trim`: String specifying the trim mode ("all" or "headers" or "fields" or :all or :headers or :fields)
- `ignore_null_bytes`: Boolean specifying if null bytes should be ignored (default: false)
- `lossy`: Boolean specifying if invalid UTF-8 characters should be replaced with a replacement character (default: false)

When `has_headers` is false, hash keys will be generated as `"c0"`, `"c1"`, etc.

## Requirements

- Ruby >= 3.1.0
- Rust toolchain (for installation from source)

## Performance

This library is faster than the standard Ruby CSV library. It's also faster than any other CSV gem I've been able to find.

Here's some unscientific benchmarks. You can find the code in the [benchmark/comparison_benchmark.rb](benchmark/comparison_benchmark.rb) file.

### 1,000,000 records

```
🏃 Running benchmarks...
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
      CSV - StringIO      0.083 (± 0.0%) i/s    (12.06 s/i) -      3.000 in  36.304469s
  FastCSV - StringIO      0.335 (± 0.0%) i/s     (2.98 s/i) -     10.000 in  31.019521s
      OSV - StringIO      0.705 (± 0.0%) i/s     (1.42 s/i) -     21.000 in  30.629511s
   CSV - Hash output      0.060 (± 0.0%) i/s    (16.74 s/i) -      2.000 in  33.475977s
   OSV - Hash output      0.434 (± 0.0%) i/s     (2.30 s/i) -     13.000 in  30.071679s
  CSV - Array output      0.063 (± 0.0%) i/s    (15.88 s/i) -      2.000 in  32.229906s
  OSV - Array output      0.406 (± 0.0%) i/s     (2.47 s/i) -     12.000 in  31.072600s
FastCSV - Array output
                          0.321 (± 0.0%) i/s     (3.11 s/i) -     10.000 in  31.458966s
OSV - Direct Open Array output
                          0.686 (± 0.0%) i/s     (1.46 s/i) -     21.000 in  30.639715s
       OSV - Gzipped      0.524 (± 0.0%) i/s     (1.91 s/i) -     16.000 in  30.695259s
OSV - Gzipped Direct      0.519 (± 0.0%) i/s     (1.93 s/i) -     16.000 in  30.830005s
   FastCSV - Gzipped      0.313 (± 0.0%) i/s     (3.20 s/i) -     10.000 in  32.031002s
       CSV - Gzipped      0.057 (± 0.0%) i/s    (17.55 s/i) -      2.000 in  35.107808s

Comparison:
      OSV - StringIO          : 0.7 i/s
OSV - Direct Open Array output: 0.7 i/s - 1.03x  slower
       OSV - Gzipped          : 0.5 i/s - 1.34x  slower
OSV - Gzipped Direct          : 0.5 i/s - 1.36x  slower
   OSV - Hash output          : 0.4 i/s - 1.62x  slower
  OSV - Array output          : 0.4 i/s - 1.74x  slower
  FastCSV - StringIO          : 0.3 i/s - 2.10x  slower
FastCSV - Array output        : 0.3 i/s - 2.20x  slower
   FastCSV - Gzipped          : 0.3 i/s - 2.26x  slower
      CSV - StringIO          : 0.1 i/s - 8.50x  slower
  CSV - Array output          : 0.1 i/s - 11.20x  slower
   CSV - Hash output          : 0.1 i/s - 11.80x  slower
       CSV - Gzipped          : 0.1 i/s - 12.37x  slower
```
