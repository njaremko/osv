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

```ruby
# Reading TSV files
OSV.for_each("path/to/file.tsv", col_sep: "\t") do |row|
  puts row["name"]
end

# Reading without headers
OSV.for_each("path/to/file.csv", has_headers: false) do |row|
  # Headers will be automatically generated as "c0", "c1", etc.
  puts row["c0"]
end
```

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

ruby 3.3.3 (2024-06-12 revision f1c7b6f435) [arm64-darwin23]
Warming up --------------------------------------
   OSV - Hash output     1.000 i/100ms
   CSV - Hash output     1.000 i/100ms
  OSV - Array output     1.000 i/100ms
  CSV - Array output     1.000 i/100ms
FastCSV - Array output
                         1.000 i/100ms
      OSV - StringIO     1.000 i/100ms
      CSV - StringIO     1.000 i/100ms
  FastCSV - StringIO     1.000 i/100ms
       OSV - Gzipped     1.000 i/100ms
       CSV - Gzipped     1.000 i/100ms
Calculating -------------------------------------
   OSV - Hash output      0.578 (± 0.0%) i/s     (1.73 s/i) -      3.000 in   5.287845s
   CSV - Hash output      0.117 (± 0.0%) i/s     (8.57 s/i) -      1.000 in   8.571770s
  OSV - Array output      1.142 (± 0.0%) i/s  (875.97 ms/i) -      5.000 in   5.234694s
  CSV - Array output      0.235 (± 0.0%) i/s     (4.25 s/i) -      2.000 in   8.561144s
FastCSV - Array output
                          0.768 (± 0.0%) i/s     (1.30 s/i) -      4.000 in   6.924574s
      OSV - StringIO      0.522 (± 0.0%) i/s     (1.91 s/i) -      3.000 in   5.803969s
      CSV - StringIO      0.132 (± 0.0%) i/s     (7.59 s/i) -      1.000 in   7.593243s
  FastCSV - StringIO      1.039 (± 0.0%) i/s  (962.53 ms/i) -      6.000 in   5.806644s
       OSV - Gzipped      0.437 (± 0.0%) i/s     (2.29 s/i) -      3.000 in   6.885125s
       CSV - Gzipped      0.115 (± 0.0%) i/s     (8.68 s/i) -      1.000 in   8.684069s

Comparison:
  OSV - Array output:        1.1 i/s
  FastCSV - StringIO:        1.0 i/s - 1.10x  slower
FastCSV - Array output:        0.8 i/s - 1.49x  slower
   OSV - Hash output:        0.6 i/s - 1.98x  slower
      OSV - StringIO:        0.5 i/s - 2.19x  slower
       OSV - Gzipped:        0.4 i/s - 2.61x  slower
  CSV - Array output:        0.2 i/s - 4.86x  slower
      CSV - StringIO:        0.1 i/s - 8.67x  slower
   CSV - Hash output:        0.1 i/s - 9.79x  slower
       CSV - Gzipped:        0.1 i/s - 9.91x  slower
```
