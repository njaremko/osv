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

### Basic Usage with Hash Output

Each row is returned as a hash where the keys are the column headers:

```ruby
require 'osv'

# Read from a file
OSV.for_each("path/to/file.csv") do |row|
  # row is a Hash like {"name" => "John", "age" => "25"}
  puts row["name"]
end

# Without a block, returns an Enumerator
rows = OSV.for_each("path/to/file.csv")
rows.each { |row| puts row["name"] }
```

### Array Output Mode

If you prefer working with arrays instead of hashes, use `for_each_compat`:

```ruby
OSV.for_each("path/to/file.csv", result_type: :array) do |row|
  # row is an Array like ["John", "25"]
  puts row[0]
end
```

### Options

Both methods support the following options:

- `has_headers`: Boolean indicating if the first row contains headers (default: true)
- `col_sep`: String specifying the field separator (default: ",")
- `quote_char`: String specifying the quote character (default: "\"")
- `nil_string`: String that should be interpreted as nil
  - by default, empty strings are interpreted as empty strings
  - if you want to interpret empty strings as nil, set this to an empty string
- `buffer_size`: Integer specifying the read buffer size
- `result_type`: String specifying the output format ("hash" or "array")
- `flexible`: Boolean specifying if the parser should be flexible (default: false)
- `flexible_default`: String specifying the default value for missing fields. Implicitly enables flexible mode if set. (default: `nil`)

### Input Sources

OSV supports reading from:

- File paths (as strings)
- IO objects
  - Important caveat: the IO object must respond to `rb_io_descriptor` with a file descriptor.
- StringIO objects
  - Note: when you do this, the string is read (in full) into a Rust string, and we parse it there.

```ruby
# From file path
OSV.for_each("path/to/file.csv") { |row| puts row["name"] }

# From IO object
File.open("path/to/file.csv") do |file|
  OSV.for_each(file) { |row| puts row["name"] }
end

# From StringIO
data = StringIO.new("name,age\nJohn,25")
OSV.for_each(data) { |row| puts row["name"] }
```

## Requirements

- Ruby >= 3.1.0
- Rust toolchain (for installation from source)

## Performance

This library is faster than the standard Ruby CSV library, and is comparable to the fastest CSV parser gems I've used.

Here's some unscientific benchmarks. You can find the code in the [benchmark/comparison_benchmark.rb](benchmark/comparison_benchmark.rb) file.

### 10,000 lines

```
Benchmarking with 10001 lines of data

ruby 3.3.3 (2024-06-12 revision f1c7b6f435) [arm64-darwin23]
Warming up --------------------------------------
   OSV - Hash output     6.000 i/100ms
   CSV - Hash output     1.000 i/100ms
  OSV - Array output    18.000 i/100ms
  CSV - Array output     2.000 i/100ms
FastCSV - Array output
                         9.000 i/100ms
      OSV - StringIO     7.000 i/100ms
      CSV - StringIO     1.000 i/100ms
  FastCSV - StringIO    20.000 i/100ms
       OSV - Gzipped     6.000 i/100ms
       CSV - Gzipped     1.000 i/100ms
Calculating -------------------------------------
   OSV - Hash output     73.360 (± 4.1%) i/s   (13.63 ms/i) -    366.000 in   5.000390s
   CSV - Hash output     11.937 (±25.1%) i/s   (83.78 ms/i) -     52.000 in   5.036297s
  OSV - Array output    189.738 (± 8.4%) i/s    (5.27 ms/i) -    954.000 in   5.071018s
  CSV - Array output     25.471 (±11.8%) i/s   (39.26 ms/i) -    120.000 in   5.015289s
FastCSV - Array output
                         97.867 (± 2.0%) i/s   (10.22 ms/i) -    495.000 in   5.060957s
      OSV - StringIO     80.784 (± 6.2%) i/s   (12.38 ms/i) -    406.000 in   5.046696s
      CSV - StringIO     15.872 (± 0.0%) i/s   (63.01 ms/i) -     80.000 in   5.043361s
  FastCSV - StringIO    200.511 (± 2.0%) i/s    (4.99 ms/i) -      1.020k in   5.088592s
       OSV - Gzipped     55.220 (±12.7%) i/s   (18.11 ms/i) -    258.000 in   5.030928s
       CSV - Gzipped     12.591 (±15.9%) i/s   (79.42 ms/i) -     59.000 in   5.039709s

Comparison:
  FastCSV - StringIO:      200.5 i/s
  OSV - Array output:      189.7 i/s - same-ish: difference falls within error
FastCSV - Array output:       97.9 i/s - 2.05x  slower
      OSV - StringIO:       80.8 i/s - 2.48x  slower
   OSV - Hash output:       73.4 i/s - 2.73x  slower
       OSV - Gzipped:       55.2 i/s - 3.63x  slower
  CSV - Array output:       25.5 i/s - 7.87x  slower
      CSV - StringIO:       15.9 i/s - 12.63x  slower
       CSV - Gzipped:       12.6 i/s - 15.92x  slower
   CSV - Hash output:       11.9 i/s - 16.80x  slower
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
   OSV - Hash output      0.502 (± 0.0%) i/s     (1.99 s/i) -      5.000 in  10.293445s
   CSV - Hash output      0.120 (± 0.0%) i/s     (8.30 s/i) -      2.000 in  16.601119s
  OSV - Array output      1.291 (±77.4%) i/s  (774.46 ms/i) -     10.000 in  10.627281s
OSV - Direct Open Array output
                          1.576 (± 0.0%) i/s  (634.38 ms/i) -     16.000 in  10.499217s
  CSV - Array output      0.193 (± 0.0%) i/s     (5.17 s/i) -      2.000 in  10.518472s
FastCSV - Array output
                          0.352 (± 0.0%) i/s     (2.84 s/i) -      4.000 in  11.378973s
      OSV - StringIO      0.500 (± 0.0%) i/s     (2.00 s/i) -      5.000 in  10.224951s
      CSV - StringIO      0.134 (± 0.0%) i/s     (7.47 s/i) -      2.000 in  15.090316s
  FastCSV - StringIO      1.047 (± 0.0%) i/s  (955.00 ms/i) -     11.000 in  10.543497s
       OSV - Gzipped      0.385 (± 0.0%) i/s     (2.60 s/i) -      4.000 in  10.458821s
       CSV - Gzipped      0.117 (± 0.0%) i/s     (8.57 s/i) -      2.000 in  17.156519s

Comparison:
OSV - Direct Open Array output:        1.6 i/s
  OSV - Array output:        1.3 i/s - same-ish: difference falls within error
  FastCSV - StringIO:        1.0 i/s - 1.51x  slower
   OSV - Hash output:        0.5 i/s - 3.14x  slower
      OSV - StringIO:        0.5 i/s - 3.15x  slower
       OSV - Gzipped:        0.4 i/s - 4.10x  slower
FastCSV - Array output:        0.4 i/s - 4.47x  slower
  CSV - Array output:        0.2 i/s - 8.15x  slower
      CSV - StringIO:        0.1 i/s - 11.78x  slower
   CSV - Hash output:        0.1 i/s - 13.08x  slower
       CSV - Gzipped:        0.1 i/s - 13.52x  slower
```
