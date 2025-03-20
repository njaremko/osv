# Changelog

## 0.5.2

- Lots of new tests
- One bug fix with extremely wide CSVs
- Do not intern headers when parsing with result type set to array

## 0.5.1

- Attempting to determine if the value being read is a `StringIO` is difficult to due safely, so just treat it as an `IO`-like object.

## 0.5.0

- Got rid of surprising behaviour that bypassed ruby if the provided IO had a file descriptor. It led to confusing bugs where people would write a custom read method that was ignored because we read the file descriptor directly.
- No longer read file into memory when reading gzipped data...
- Cleanup the reader implementation in general

## 0.4.4

- Added support for cross-compilation for multiple platforms

## 0.4.2 and 0.4.3

- Fix occasional segfault when parsing with `result_type: :hash`

## 0.4.1

- Fix bug with lossy not being respected when parsing headers

## 0.4.0

- Added `lossy` option to `for_each` that allows replacing invalid UTF-8 characters with a replacement character
- Removed `flexible_default` option from `for_each`

## 0.3.21

- Fix bug where `ignore_null_bytes` was not being respected in enumerators.

## 0.3.19 and 0.3.20

- Added `ignore_null_bytes` option to `for_each` that allows ignoring null bytes in fields
- The latter just removes an unneeded string copy when filtering out null bytes

## 0.3.18

- Fix handling of passing in explicit nil for optional arguments.

## 0.3.17

- Remove multi-threaded parsing. It was a bad idea. Performance is better without it. Code is simpler.

## 0.3.16

- Optimize hash construction by interning key strings

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
