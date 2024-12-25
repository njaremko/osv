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

### 10,000 lines

```
Benchmarking with 100001 lines of data

ruby 3.3.6 (2024-11-05 revision 75015d4c1f) [arm64-darwin24]
Warming up --------------------------------------
   OSV - Hash output     1.000 i/100ms
   CSV - Hash output     1.000 i/100ms
  OSV - Array output     1.000 i/100ms
OSV - Direct Open Array output
                        12.719M i/100ms
  CSV - Array output     1.000 i/100ms
FastCSV - Array output
                         1.000 i/100ms
      OSV - StringIO     1.000 i/100ms
      CSV - StringIO     1.000 i/100ms
  FastCSV - StringIO     1.000 i/100ms
       OSV - Gzipped     1.000 i/100ms
       CSV - Gzipped     1.000 i/100ms
Calculating -------------------------------------
   OSV - Hash output      6.722 (±14.9%) i/s  (148.77 ms/i) -     59.000 in  10.074753s
   CSV - Hash output      1.223 (± 0.0%) i/s  (817.62 ms/i) -     13.000 in  10.788284s
  OSV - Array output     17.284 (±11.6%) i/s   (57.86 ms/i) -    171.000 in  10.007321s
OSV - Direct Open Array output
                        213.629M (±13.5%) i/s    (4.68 ns/i) -      1.921B in  10.005506s
  CSV - Array output      2.193 (± 0.0%) i/s  (455.93 ms/i) -     22.000 in  10.052607s
FastCSV - Array output
                          7.993 (± 0.0%) i/s  (125.11 ms/i) -     80.000 in  10.053729s
      OSV - StringIO      6.626 (±15.1%) i/s  (150.91 ms/i) -     66.000 in  10.103646s
      CSV - StringIO      1.478 (± 0.0%) i/s  (676.78 ms/i) -     15.000 in  10.158640s
  FastCSV - StringIO     17.074 (± 5.9%) i/s   (58.57 ms/i) -    171.000 in  10.059266s
       OSV - Gzipped      5.639 (± 0.0%) i/s  (177.32 ms/i) -     57.000 in  10.152487s
       CSV - Gzipped      1.176 (± 0.0%) i/s  (850.19 ms/i) -     12.000 in  10.233398s

Comparison:
OSV - Direct Open Array output: 213629268.6 i/s
  OSV - Array output:       17.3 i/s - 12360250.79x  slower
  FastCSV - StringIO:       17.1 i/s - 12511956.50x  slower
FastCSV - Array output:        8.0 i/s - 26727225.72x  slower
   OSV - Hash output:        6.7 i/s - 31780615.83x  slower
      OSV - StringIO:        6.6 i/s - 32239620.60x  slower
       OSV - Gzipped:        5.6 i/s - 37881517.48x  slower
  CSV - Array output:        2.2 i/s - 97400427.87x  slower
      CSV - StringIO:        1.5 i/s - 144580048.04x  slower
   CSV - Hash output:        1.2 i/s - 174666591.31x  slower
       CSV - Gzipped:        1.2 i/s - 181626018.23x  slower
```

### 1,000,000 lines

```
Benchmarking with 1000001 lines of data

ruby 3.3.6 (2024-11-05 revision 75015d4c1f) [arm64-darwin24]
Warming up --------------------------------------
   OSV - Hash output     1.000 i/100ms
   CSV - Hash output     1.000 i/100ms
  OSV - Array output     1.000 i/100ms
OSV - Direct Open Array output
                         1.000 i/100ms
  CSV - Array output     1.000 i/100ms
FastCSV - Array output
                         1.000 i/100ms
      OSV - StringIO     1.000 i/100ms
      CSV - StringIO     1.000 i/100ms
  FastCSV - StringIO     1.000 i/100ms
       OSV - Gzipped     1.000 i/100ms
       CSV - Gzipped     1.000 i/100ms
Calculating -------------------------------------
   OSV - Hash output      0.492 (± 0.0%) i/s     (2.03 s/i) -      5.000 in  10.463278s
   CSV - Hash output      0.114 (± 0.0%) i/s     (8.75 s/i) -      2.000 in  17.573877s
  OSV - Array output      1.502 (± 0.0%) i/s  (665.58 ms/i) -     14.000 in  10.217551s
OSV - Direct Open Array output
                          1.626 (± 0.0%) i/s  (614.90 ms/i) -     16.000 in  10.190323s
  CSV - Array output      0.183 (± 0.0%) i/s     (5.46 s/i) -      2.000 in  10.951943s
FastCSV - Array output
                          0.326 (± 0.0%) i/s     (3.07 s/i) -      4.000 in  12.340605s
      OSV - StringIO      0.567 (± 0.0%) i/s     (1.76 s/i) -      6.000 in  10.698027s
      CSV - StringIO      0.141 (± 0.0%) i/s     (7.10 s/i) -      2.000 in  14.237144s
  FastCSV - StringIO      0.923 (± 0.0%) i/s     (1.08 s/i) -     10.000 in  11.567775s
       OSV - Gzipped      0.437 (± 0.0%) i/s     (2.29 s/i) -      5.000 in  11.452764s
       CSV - Gzipped      0.104 (± 0.0%) i/s     (9.64 s/i) -      2.000 in  19.373423s

Comparison:
OSV - Direct Open Array output:        1.6 i/s
  OSV - Array output:        1.5 i/s - 1.08x  slower
  FastCSV - StringIO:        0.9 i/s - 1.76x  slower
      OSV - StringIO:        0.6 i/s - 2.87x  slower
   OSV - Hash output:        0.5 i/s - 3.30x  slower
       OSV - Gzipped:        0.4 i/s - 3.72x  slower
FastCSV - Array output:        0.3 i/s - 4.99x  slower
  CSV - Array output:        0.2 i/s - 8.88x  slower
      CSV - StringIO:        0.1 i/s - 11.55x  slower
   CSV - Hash output:        0.1 i/s - 14.24x  slower
       CSV - Gzipped:        0.1 i/s - 15.68x  slower
```
