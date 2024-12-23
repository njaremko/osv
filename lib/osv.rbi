# typed: strict

module OSV
  sig do
    params(
      input: T.any(String, StringIO, IO),
      has_headers: T.nilable(T::Boolean),
      col_sep: T.nilable(String),
      quote_char: T.nilable(String),
      nil_string: T.nilable(String),
      buffer_size: T.nilable(Integer),
      result_type: T.nilable(String),
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
    &blk
  )
  end
end
