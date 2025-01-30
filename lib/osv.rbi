# typed: strict

module OSV
  # Options:
  #   - `has_headers`: Boolean indicating if the first row contains headers
  #                    (default: true)
  #   - `col_sep`: String specifying the field separator
  #                (default: ",")
  #   - `quote_char`: String specifying the quote character
  #                   (default: "\"")
  #   - `nil_string`: String that should be interpreted as nil
  #                   By default, empty strings are interpreted as empty strings.
  #                   If you want to interpret empty strings as nil, set this to
  #                   an empty string.
  #   - `buffer_size`: Integer specifying the read buffer size
  #   - `result_type`: String specifying the output format
  #                    ("hash" or "array" or :hash or :array)
  #   - `flexible`: Boolean specifying if the parser should be flexible
  #                 (default: false)
  #   - `trim`: String specifying the trim mode
  #             ("all" or "headers" or "fields" or :all or :headers or :fields)
  #             (default: `nil`)
  #   - `ignore_null_bytes`: Boolean specifying if null bytes should be ignored
  #                         (default: false)
  #   - `lossy`: Boolean specifying if invalid UTF-8 characters should be replaced with a replacement character
  sig do
    params(
      input: T.any(String, StringIO, IO),
      has_headers: T.nilable(T::Boolean),
      col_sep: T.nilable(String),
      quote_char: T.nilable(String),
      nil_string: T.nilable(String),
      buffer_size: T.nilable(Integer),
      result_type: T.nilable(T.any(String, Symbol)),
      flexible: T.nilable(T::Boolean),
      ignore_null_bytes: T.nilable(T::Boolean),
      trim: T.nilable(T.any(String, Symbol)),
      blk: T.nilable(T.proc.params(row: T.any(T::Hash[String, T.nilable(String)], T::Array[T.nilable(String)])).void)
    ).returns(T.any(Enumerator, T.untyped))
  end
  def self.for_each(
    input,
    has_headers: true,
    col_sep: nil,
    quote_char: nil,
    nil_string: nil,
    buffer_size: nil,
    result_type: nil,
    flexible: nil,
    ignore_null_bytes: nil,
    trim: nil,
    lossy: nil,
    &blk
  )
  end
end
