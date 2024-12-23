# typed: strict

module OSV
  sig do
    type_parameters(:T)
      .params(
        input: T.any(String, StringIO, IO),
        has_headers: T.nilable(T::Boolean),
        delimiter: T.nilable(String),
        blk: T.proc.params(row: T::Hash[String, String]).void
      )
      .returns(T.untyped)
  end
  def self.for_each(input, has_headers: true, delimiter: nil, &blk)
  end

  sig do
    type_parameters(:T)
      .params(
        input: T.any(String, StringIO, IO),
        has_headers: T.nilable(T::Boolean),
        delimiter: T.nilable(String),
        blk: T.proc.params(row: T::Array[String]).void
      )
      .returns(T.untyped)
  end
  def self.for_each_compat(input, has_headers: true, delimiter: nil, &blk)
  end
end
