# Changelog

## 0.3.3

- Added support for gzip files

## 0.3.2

- Intern strings used as keys in hashes until no longer referenced by Ruby to get rid of extra allocations

## 0.3.0

- Got rid of `for_each_compat`. Now use `for_each(result_type: "array")` or `for_each(result_type: :array)`
- Added `result_type` option to `parse_csv`
- Added `buffer_size` option to `parse_csv`
