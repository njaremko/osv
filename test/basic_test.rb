# frozen_string_literal: true

require "osv"
require "zlib"
require "minitest/autorun"

class BasicTest < Minitest::Test
  def test_parse_csv_with_headers
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    OSV.for_each("test/test.csv") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_with_headers_null
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => nil, "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    OSV.for_each("test/test.csv", nil_string: "Jane") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_with_tsv
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    OSV.for_each("test/test.tsv", col_sep: "\t") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_without_headers
    expected = [
      { "c0" => "id", "c1" => "name", "c2" => "age" },
      { "c1" => "John", "c2" => "25", "c0" => "1" },
      { "c1" => "Jane", "c2" => "30", "c0" => "2" },
      { "c0" => "3", "c1" => "Jim", "c2" => "35" }
    ]
    actual = []
    OSV.for_each("test/test.csv", has_headers: false) { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_with_io
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    File.open("test/test.csv") { |file| OSV.for_each(file) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_gzip
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    File.open("test/test.csv.gz", "wb") do |gz_file|
      gz = Zlib::GzipWriter.new(gz_file)
      gz.write(File.read("test/test.csv"))
      gz.close
    end
    OSV.for_each("test/test.csv.gz") { |row| actual << row }
    assert_equal expected, actual
  ensure
    FileUtils.rm_f("test/test.csv.gz")
  end

  def test_parse_csv_with_string_io
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    csv_data = File.read("test/test.csv")
    string_io = StringIO.new(csv_data)
    OSV.for_each(string_io) { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_compat_without_headers
    expected = [%w[id name age], %w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    actual = []
    OSV.for_each("test/test.csv", has_headers: false, result_type: "array") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_compat_with_headers
    expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    actual = []
    OSV.for_each("test/test.csv", has_headers: true, result_type: "array") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_compat_with_headers_null
    expected = [%w[1 John 25], ["2", nil, "30"], %w[3 Jim 35]]
    actual = []
    OSV.for_each("test/test.csv", has_headers: true, nil_string: "Jane", result_type: "array") { |row| actual << row }
    assert_equal expected, actual
  end

  def test_parse_csv_with_empty_field
    Tempfile.create(%w[test .csv]) do |tempfile|
      # Copy existing content and add a line with empty field
      content = File.read("test/test.csv")
      content += "4,,40\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "id" => "1", "age" => "25", "name" => "John" },
        { "name" => "Jane", "id" => "2", "age" => "30" },
        { "name" => "Jim", "age" => "35", "id" => "3" },
        { "id" => "4", "name" => "", "age" => "40" }
      ]
      actual = []
      OSV.for_each(tempfile.path) { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_empty_field_as_nil_string
    Tempfile.create(%w[test .csv]) do |tempfile|
      # Copy existing content and add a line with empty field
      content = File.read("test/test.csv")
      content += "4,,40\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "id" => "1", "age" => "25", "name" => "John" },
        { "name" => "Jane", "id" => "2", "age" => "30" },
        { "name" => "Jim", "age" => "35", "id" => "3" },
        { "id" => "4", "name" => nil, "age" => "40" }
      ]
      actual = []
      OSV.for_each(tempfile.path, nil_string: "") { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_default_strict
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "id" => "1", "age" => "25", "name" => "John" },
        { "name" => "Jane", "id" => "2", "age" => "30" },
        { "name" => "Jim", "age" => "35", "id" => "3" }
      ]
      actual = []
      OSV.for_each(tempfile.path) { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "id" => "1", "age" => "25", "name" => "John" },
        { "name" => "Jane", "id" => "2", "age" => "30" },
        { "name" => "Jim", "age" => "35", "id" => "3" },
        { "id" => "4", "name" => "oops", "age" => nil }
      ]
      actual = []
      OSV.for_each(tempfile.path, flexible: true) { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible_without_headers
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "c2" => "age", "c0" => "id", "c1" => "name" },
        { "c2" => "25", "c0" => "1", "c1" => "John" },
        { "c1" => "Jane", "c2" => "30", "c0" => "2" },
        { "c0" => "3", "c2" => "35", "c1" => "Jim" },
        { "c1" => "oops", "c0" => "4", "c2" => nil }
      ]
      actual = []
      OSV.for_each(tempfile.path, has_headers: false, flexible: true) { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible_default
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "id" => "1", "age" => "25", "name" => "John" },
        { "name" => "Jane", "id" => "2", "age" => "30" },
        { "name" => "Jim", "age" => "35", "id" => "3" },
        { "id" => "4", "name" => "oops", "age" => "" }
      ]
      actual = []
      OSV.for_each(tempfile.path, flexible_default: "") { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible_default_without_headers
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [
        { "c0" => "id", "c1" => "name", "c2" => "age" },
        { "c1" => "John", "c0" => "1", "c2" => "25" },
        { "c1" => "Jane", "c2" => "30", "c0" => "2" },
        { "c0" => "3", "c1" => "Jim", "c2" => "35" },
        { "c0" => "4", "c2" => "", "c1" => "oops" }
      ]
      actual = []
      OSV.for_each(tempfile.path, has_headers: false, flexible_default: "") { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible_array
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35], %w[4 oops]]
      actual = []
      OSV.for_each(tempfile.path, flexible: true, result_type: :array) { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_with_missing_field_flexible_default_array
    Tempfile.create(%w[test .csv]) do |tempfile|
      content = File.read("test/test.csv")
      content += "4,oops\n"
      tempfile.write(content)
      tempfile.close

      expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35], ["4", "oops", ""]]
      actual = []
      OSV.for_each(tempfile.path, flexible_default: "", result_type: "array") { |row| actual << row }
      assert_equal expected, actual
    end
  end

  def test_parse_csv_compat_with_io_and_headers
    expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    actual = []
    File.open("test/test.csv") { |file| OSV.for_each(file, result_type: "array") { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_compat_with_io_without_headers
    expected = [%w[id name age], %w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    actual = []
    File.open("test/test.csv") do |file|
      OSV.for_each(file, has_headers: false, result_type: "array") { |row| actual << row }
    end
    assert_equal expected, actual
  end

  def test_for_each_without_block
    result = OSV.for_each("test/test.csv")
    assert_instance_of Enumerator, result
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    assert_equal expected, result.to_a
  end

  def test_for_each_compat_without_block
    result = OSV.for_each("test/test.csv", result_type: "array")
    assert_instance_of Enumerator, result
    expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    assert_equal expected, result.to_a
  end

  def test_for_each_compat_without_block_with_symbol
    result = OSV.for_each("test/test.csv", result_type: :array)
    assert_instance_of Enumerator, result
    expected = [%w[1 John 25], %w[2 Jane 30], %w[3 Jim 35]]
    assert_equal expected, result.to_a
  end

  def test_for_each_trim_all
    csv_content = <<~CSV
      id , name , age
      1 , John , 25
      2 , Jane , 30
      3 , Jim , 35
    CSV

    expected = [
      { "id" => "1", "name" => "John", "age" => "25" },
      { "id" => "2", "name" => "Jane", "age" => "30" },
      { "id" => "3", "name" => "Jim", "age" => "35" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io, trim: "all") { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_for_each_trim_headers
    csv_content = <<~CSV
      id , name , age
      1, John, 25
      2, Jane, 30
      3, Jim, 35
    CSV

    expected = [
      { "id" => "1", "name" => " John", "age" => " 25" },
      { "id" => "2", "name" => " Jane", "age" => " 30" },
      { "id" => "3", "name" => " Jim", "age" => " 35" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io, trim: :headers) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_for_each_trim_fields
    csv_content = <<~CSV
      id,name,age
      1 , John , 25
      2 , Jane , 30
      3 , Jim , 35
    CSV

    expected = [
      { "id" => "1", "name" => "John", "age" => "25" },
      { "id" => "2", "name" => "Jane", "age" => "30" },
      { "id" => "3", "name" => "Jim", "age" => "35" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io, trim: "fields") { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_in_multiple_threads
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]

    threads =
      100.times.map do
        Thread.new do
          result = OSV.for_each("test/test.csv").to_a
          assert_equal expected, result
        end
      end

    threads.each(&:join)
  end

  def test_parse_csv_in_multiple_threads_block
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]

    threads =
      100.times.map do
        Thread.new do
          results = []
          OSV.for_each("test/test.csv") { |row| results << row }
          assert_equal expected, results
        end
      end

    threads.each(&:join)
  end

  def test_parse_csv_with_gzip_io
    expected = [
      { "id" => "1", "age" => "25", "name" => "John" },
      { "name" => "Jane", "id" => "2", "age" => "30" },
      { "name" => "Jim", "age" => "35", "id" => "3" }
    ]
    actual = []
    File.open("test/test2.csv.gz", "wb") do |gz_file|
      gz = Zlib::GzipWriter.new(gz_file)
      gz.write(File.read("test/test.csv"))
      gz.close
    end
    Zlib::GzipReader.open("test/test2.csv.gz") { |gz| OSV.for_each(gz) { |row| actual << row } }
    assert_equal expected, actual
  ensure
    FileUtils.rm_f("test/test2.csv.gz")
  end

  def test_parse_csv_with_quoted_commas
    csv_content = <<~CSV
      id,name,description
      1,"Smith, John","Manager, Sales"
      2,"Doe, Jane","Director, HR"
    CSV

    expected = [
      { "id" => "1", "name" => "Smith, John", "description" => "Manager, Sales" },
      { "id" => "2", "name" => "Doe, Jane", "description" => "Director, HR" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_escaped_quotes
    csv_content = <<~CSV
      id,name,quote
      1,"John","He said ""Hello World"""
      2,"Jane","She replied ""Hi there!"""
    CSV

    expected = [
      { "id" => "1", "name" => "John", "quote" => 'He said "Hello World"' },
      { "id" => "2", "name" => "Jane", "quote" => 'She replied "Hi there!"' }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_newlines_in_quotes
    csv_content = <<~CSV
      id,name,address
      1,"John Smith","123 Main St.
      Apt 4B
      New York, NY"
      2,"Jane Doe","456 Park Ave.
      Suite 789"
    CSV

    expected = [
      { "id" => "1", "name" => "John Smith", "address" => "123 Main St.\nApt 4B\nNew York, NY" },
      { "id" => "2", "name" => "Jane Doe", "address" => "456 Park Ave.\nSuite 789" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_unicode
    csv_content = <<~CSV
      id,name,description
      1,"José García","Señor developer 👨‍💻"
      2,"Zoë Smith","⭐ Project lead"
    CSV

    expected = [
      { "id" => "1", "name" => "José García", "description" => "Señor developer 👨‍💻" },
      { "id" => "2", "name" => "Zoë Smith", "description" => "⭐ Project lead" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_bom
    csv_content = "\xEF\xBB\xBF" + <<~CSV
      id,name,age
      1,John,25
      2,Jane,30
    CSV

    expected = [{ "id" => "1", "name" => "John", "age" => "25" }, { "id" => "2", "name" => "Jane", "age" => "30" }]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_mixed_line_endings
    csv_content = "id,name,age\r\n1,John,25\n2,Jane,30\r\n3,Jim,35"

    expected = [
      { "id" => "1", "name" => "John", "age" => "25" },
      { "id" => "2", "name" => "Jane", "age" => "30" },
      { "id" => "3", "name" => "Jim", "age" => "35" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_empty_lines
    csv_content = <<~CSV
      id,name,age

      1,John,25

      2,Jane,30

      3,Jim,35

    CSV

    expected = [
      { "id" => "1", "name" => "John", "age" => "25" },
      { "id" => "2", "name" => "Jane", "age" => "30" },
      { "id" => "3", "name" => "Jim", "age" => "35" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_long_line
    long_text = "x" * 1_000_000
    csv_content = <<~CSV
      id,name,description
      1,John,#{long_text}
      2,Jane,Short description
    CSV

    expected = [
      { "id" => "1", "name" => "John", "description" => long_text },
      { "id" => "2", "name" => "Jane", "description" => "Short description" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_explicit_nil_kwargs
    csv_content = <<~CSV
      id,name,age
      1,John,25
      2,Jane,30
    CSV

    expected = [{ "id" => "1", "name" => "John", "age" => "25" }, { "id" => "2", "name" => "Jane", "age" => "30" }]

    actual = []
    StringIO
      .new(csv_content)
      .tap do |io|
        OSV.for_each(
          io,
          has_headers: nil,
          col_sep: nil,
          quote_char: nil,
          nil_string: nil,
          result_type: nil,
          flexible: nil,
          flexible_default: nil,
          trim: nil
        ) { |row| actual << row }
      end
    assert_equal expected, actual
  end

  def test_parse_csv_with_whitespace_and_quotes
    csv_content = <<~CSV
      id,name,description
      1,  John  ,  unquoted spaces
      2," Jane ",  "  quoted spaces  "
      3,"Jim","  mixed  "
    CSV

    expected = [
      { "id" => "1", "description" => "  unquoted spaces", "name" => "  John  " },
      { "id" => "2", "description" => "  \"  quoted spaces  \"", "name" => " Jane " },
      { "id" => "3", "description" => "  mixed  ", "name" => "Jim" }
    ]
    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_empty_quoted_vs_unquoted
    csv_content = <<~CSV
      id,quoted,unquoted
      1,"",
      2,," "
      3,,
      4,"  ",
    CSV

    expected = [
      { "id" => "1", "quoted" => "", "unquoted" => "" },
      { "id" => "2", "quoted" => "", "unquoted" => " " },
      { "id" => "3", "quoted" => "", "unquoted" => "" },
      { "id" => "4", "quoted" => "  ", "unquoted" => "" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_duplicate_headers
    csv_content = <<~CSV
      id,name,id,name
      1,John,A,Johnny
      2,Jane,B,Janet
    CSV

    expected = [%w[1 John A Johnny], %w[2 Jane B Janet]]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io, result_type: :array) { |row| actual << row } }
    assert_equal expected, actual
  end

  def test_parse_csv_with_null_bytes
    csv_content = <<~CSV
      id,na\0me,description
      1,Jo\0hn,test
      2,Jane,te\0st
    CSV

    expected = [
      { "id" => "1", "name" => "John", "description" => "test" },
      { "id" => "2", "name" => "Jane", "description" => "test" }
    ]

    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io, ignore_null_bytes: true) { |row| actual << row } }
    assert_equal expected, actual

    actual = OSV.for_each(StringIO.new(csv_content), ignore_null_bytes: true).to_a
    assert_equal expected, actual

    # Without ignore_null_bytes, null bytes are preserved
    actual = []
    StringIO.new(csv_content).tap { |io| OSV.for_each(io) { |row| actual << row } }
    assert_equal [
      { "id" => "1", "na\0me" => "Jo\0hn", "description" => "test" },
      { "id" => "2", "na\0me" => "Jane", "description" => "te\0st" }
    ], actual
  end
end
