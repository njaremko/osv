# Changelog

## 0.3.15

- Some internal refactoring to improve maintainability
- More optimizations for parsing IO-like objects without an underlying file handle

## 0.3.14

After quite a bit of profiling:

- When you give `OSV` a file handle IO object, we have an optimization to grab the underlying open file handle and do all reading directly in Rust. This release adds lots of optimizations for parsing objects that implement `IO`'s `read` method without having an underlying file handle available.
- This release adds a lot of optimizations for parsing `StringIO` objects, as well as anything that doesn't implement `IO`'s `read` method, but does implement `to_str` or `to_s` methods.
- Further optimizations to string allocations in Rust code.

## 0.3.13

- Turns out, gemspec descriptions cannot be markdown. Fixing that.

## 0.3.12

- Attempt at improving RubyGems page for the gem

## 0.3.11

- Set license to MIT in gemspec

## 0.3.10

- Added `trim` option to `for_each` that allows trimming of fields and headers

## 0.3.9

- Some optimizations, and a fix for a bug where file handles weren't being closed

## 0.3.8

- Added `flexible` option to `for_each` that allows flexible parsing of CSV files without a default value

## 0.3.7

- Added `flexible_default` option to `for_each` that allows flexible parsing of CSV files when set to a string. Defaults to `nil`.

## 0.3.6

- Fix bug introduced in 0.3.5 where `nil_string` was not being parsed correctly

## 0.3.5

- `nil_string` no longer defaults to an empty string. It now defaults to `nil`. Which means that empty strings are interpreted as empty strings.

## 0.3.4

- Added support for handling non-file backed IO objects in single threaded mode
- General refactoring to improve performance and reduce allocations

## 0.3.3

- Added support for gzip files

## 0.3.2

- Intern strings used as keys in hashes until no longer referenced by Ruby to get rid of extra allocations

## 0.3.0

- Got rid of `for_each_compat`. Now use `for_each(result_type: "array")` or `for_each(result_type: :array)`
- Added `result_type` option to `parse_csv`
- Added `buffer_size` option to `parse_csv`
