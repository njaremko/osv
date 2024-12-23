# OSV

[![Gem Version](https://badge.fury.io/rb/osv.svg)](https://badge.fury.io/rb/osv)

OSV is a high-performance CSV parser for Ruby, implemented in Rust. It wraps BurntSushi's excellent [csv-rs](https://github.com/BurntSushi/rust-csv) crate.

It provides a simple interface for reading CSV files with support for both hash-based and array-based row formats.

The array-based mode is faster than the hash-based mode, so if you don't need the hash keys, use the array-based mode.

I have yet to figure out how to get rust to accept an implementation of this as one method with different return types, so I've had to implement two methods.

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
OSV.for_each_compat("path/to/file.csv") do |row|
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
