# Changelog

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
