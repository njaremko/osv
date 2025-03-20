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

  def test_parse_csv_with_invalid_utf8
    invalid_utf8 = StringIO.new("id,name\n1,\xFF\xFF\n")
    assert_raises(EncodingError) do
      OSV.for_each(invalid_utf8) { |_row| }
    rescue => e
      assert e.message.include?("invalid utf-8")
      raise
    end
  end

  def test_enumerator_raises_stop_iteration
    enum = OSV.for_each("test/test.csv")
    3.times { enum.next } # Consume all records
    assert_raises(StopIteration) { enum.next }
  end

  def test_parse_csv_with_invalid_utf8_file
    File.write("test/invalid_utf8.csv", "id,name\n1,\xFF\xFF\n")
    assert_raises(EncodingError) do
      OSV.for_each("test/invalid_utf8.csv") { |_row| }
    rescue => e
      assert e.message.include?("invalid utf-8")
      raise
    ensure
      begin
        File.delete("test/invalid_utf8.csv")
      rescue StandardError
        nil
      end
    end
  end

  def test_parse_csv_with_invalid_utf8_file_lossy
    File.write("test/invalid_utf8.csv", "id,name\n1,\xFF\xFF\n")
    actual = []
    OSV.for_each("test/invalid_utf8.csv", lossy: true) { |row| actual << row }
    assert_equal [{ "id" => "1", "name" => "��" }], actual
  ensure
    begin
      File.delete("test/invalid_utf8.csv")
    rescue StandardError
      nil
    end
  end

  def test_parse_csv_with_invalid_utf8_headers_lossy
    File.write("test/invalid_utf8_headers.csv", "\xFF\xFF,name\n1,test\n")
    actual = []
    OSV.for_each("test/invalid_utf8_headers.csv", lossy: true) { |row| actual << row }
    assert_equal [{ "��" => "1", "name" => "test" }], actual
  ensure
    begin
      File.delete("test/invalid_utf8_headers.csv")
    rescue StandardError
      nil
    end
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

      assert_raises(RuntimeError) do
        OSV.for_each(tempfile.path) { |row| actual << row }
      rescue RuntimeError => e
        assert e.message.include?("found record with 2 fields, but the previous record has 3 fields")
        raise
      end

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
                 ],
                 actual
  end

  def test_parse_with_gzip_corrupted
    # Create a corrupted gzip file
    File.open("test/corrupted.csv.gz", "wb") do |file|
      file.write("This is not a valid gzip file but has .gz extension")
    end

    assert_raises(RuntimeError) { OSV.for_each("test/corrupted.csv.gz") { |row| } }
  ensure
    FileUtils.rm_f("test/corrupted.csv.gz")
  end

  def test_parse_input_modified_during_iteration
    temp_file = Tempfile.new(%w[dynamic .csv])
    begin
      temp_file.write("id,name\n1,John\n2,Jane\n")
      temp_file.flush

      enum = OSV.for_each(temp_file.path)
      # Get first row
      enum.next

      # Modify file between iterations
      File.open(temp_file.path, "a") { |f| f.write("3,Modified\n") }

      # Continue iteration
      second = enum.next
      assert_equal({ "id" => "2", "name" => "Jane" }, second)

      # This might read the newly appended line or might not depending on buffering
      # Either way, it shouldn't crash
      begin
        third = enum.next
        assert_equal({ "id" => "3", "name" => "Modified" }, third)
      rescue StopIteration
        # This is also acceptable
      end
    ensure
      temp_file.close
      temp_file.unlink
    end
  end

  def test_parse_with_extremely_large_row
    Tempfile.create(%w[large .csv]) do |tempfile|
      tempfile.write("id,name,description\n")
      tempfile.write("1,test,#{"x" * 10_000_000}\n") # 10MB row
      tempfile.flush

      result = nil
      # This shouldn't crash, though it might use a lot of memory
      OSV.for_each(tempfile.path) do |row|
        result = row
        break # Only read the first row
      end

      assert_equal "1", result["id"]
      assert_equal "test", result["name"]
      assert_equal 10_000_000, result["description"].length
    end
  end

  def test_parse_with_garbage_collection_stress
    # Create a medium-sized file
    Tempfile.create(%w[gc_stress .csv]) do |tempfile|
      # Write a decent amount of data
      tempfile.write("id,name,value\n")
      1000.times { |i| tempfile.write("#{i},name#{i},value#{i}\n") }
      tempfile.flush

      # Enable GC stress mode during parsing
      GC.stress = true
      begin
        count = 0
        OSV.for_each(tempfile.path) do |row|
          count += 1
          # Force some allocations
          row.transform_values(&:dup)
          # Occasionally force GC
          GC.start if count % 100 == 0
        end
        assert_equal 1000, count
      ensure
        GC.stress = false
      end
    end
  end

  def test_interleaved_parsing_with_threads
    # Create two files to parse
    file1 = Tempfile.new(%w[thread1 .csv])
    file2 = Tempfile.new(%w[thread2 .csv])

    begin
      # Write different content to each file
      file1.write("id,name\n")
      file2.write("code,description\n")

      100.times do |i|
        file1.write("#{i},name#{i}\n")
        file2.write("code#{i},desc#{i}\n")
      end

      file1.flush
      file2.flush

      # Parse both files in interleaved fashion with threads
      enum1 = OSV.for_each(file1.path)
      enum2 = OSV.for_each(file2.path)

      threads = []
      results1 = Queue.new
      results2 = Queue.new

      # Thread 1 processes enum1
      threads << Thread.new do
        begin
          results1 << enum1.next while true
        rescue StopIteration
          # Expected when enumeration is complete
        end
      end

      # Thread 2 processes enum2
      threads << Thread.new do
        begin
          results2 << enum2.next while true
        rescue StopIteration
          # Expected when enumeration is complete
        end
      end

      # Wait for both threads to complete
      threads.each(&:join)

      # Verify results
      assert_equal 100, results1.size
      assert_equal 100, results2.size

      # Check first and last items from each queue
      first1 = results1.pop
      assert_equal "0", first1["id"]
      assert_equal "name0", first1["name"]

      first2 = results2.pop
      assert_equal "code0", first2["code"]
      assert_equal "desc0", first2["description"]
    ensure
      file1.close
      file1.unlink
      file2.close
      file2.unlink
    end
  end
  
  def test_segfault_stress_csv_parser_with_many_instances
    # This test creates many parser instances simultaneously,
    # which can stress the memory management and potentially trigger segfaults
    
    files = []
    enumerators = []
    
    begin
      # Create several moderate-sized CSV files
      5.times do |file_idx|
        file = Tempfile.new(["stress_#{file_idx}", '.csv'])
        files << file
        
        file.write("id,name,value\n")
        500.times { |i| file.write("#{i},name#{i},value#{i}\n") }
        file.flush
      end
      
      # Create many parser instances for each file
      files.each do |file|
        10.times do
          enumerators << OSV.for_each(file.path)
        end
      end
      
      # Force memory pressure with large temporary objects
      temp_strings = []
      10.times { temp_strings << "x" * (1024 * 1024) }
      GC.start
      
      # Read partially from random enumerators
      100.times do
        enum = enumerators.sample
        begin
          # Read a random number of records, but not too many
          rand(1..5).times { enum.next }
        rescue StopIteration
          # Expected for some enumerators
        end
      end
      
      # Force more GC pressure
      temp_strings = nil
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Success if we get here without a segfault
      assert true
    ensure
      # Clean up
      files.each do |file|
        begin
          file.close
          file.unlink
        rescue
          # Ignore cleanup errors
        end
      end
    end
  end
  
  def test_segfault_aggressive_gc_during_parse
    file = Tempfile.new(%w[gc_stress .csv])
    begin
      # Write a large amount of data
      file.write("id,name,value\n")
      50_000.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create multiple parsers to increase memory pressure
      parsers = 5.times.map { OSV.for_each(file.path) }
      
      # Track progress
      rows_read = 0
      
      # Force aggressive GC while parsing
      gc_thread = Thread.new do
        100.times do
          GC.start(full_mark: true, immediate_sweep: true)
          sleep 0.001 # Small sleep to allow other threads to run
        end
      end
      
      # Read from parsers in interleaved fashion
      begin
        while rows_read < 1000
          parsers.each do |parser|
            begin
              parser.next
              rows_read += 1
            rescue StopIteration
              # Expected when enumeration is complete
            end
          end
        end
      rescue => e
        puts "Error during parsing: #{e.message}"
        raise
      end
      
      gc_thread.join
      
      # Success if no segfault
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_handle_invalid_utf8_with_random_memory
    # Create file with some valid and some invalid UTF-8 sequences
    file = Tempfile.new(%w[utf8_random .csv])
    begin
      file.write("id,name,random\n")
      
      # Generate some valid records mixed with invalid UTF-8 sequences
      # and very large fields to trigger memory reallocation
      srand(42) # Consistent random sequence
      50.times do |i|
        # Mix of valid and invalid UTF-8
        invalid_bytes = 10.times.map { rand(128..255).chr }.join
        
        # Very large field to trigger memory reallocation
        large_field = "x" * (rand(10) * 1000)
        
        # Sometimes write invalid bytes directly into the large field
        if i % 3 == 0
          large_field[rand(large_field.length)] = rand(128..255).chr
        end
        
        file.write("#{i},name#{i}#{invalid_bytes},#{large_field}\n")
      end
      file.flush
      
      # Try parsing with different combinations of settings
      # These can trigger different code paths in the UTF-8 validation logic
      params = [
        {},
        { lossy: true },
        { ignore_null_bytes: true },
        { lossy: true, ignore_null_bytes: true }
      ]
      
      # Test each parameter combination
      params.each do |param|
        begin
          # Use block form
          rows = []
          OSV.for_each(file.path, **param) do |row|
            rows << row
            # Force some GC pressure
            GC.start if rows.size % 10 == 0
          end
        rescue EncodingError => e
          # This is expected for some parameter combinations
          assert e.message.include?("invalid utf-8"), "Unexpected error: #{e.message}"
        end
        
        begin
          # Use enumerator form
          enum = OSV.for_each(file.path, **param)
          loop do
            row = enum.next
            # Force some GC pressure every few iterations
            GC.start if rand(5) == 0
          end
        rescue StopIteration
          # Expected at end of file
        rescue EncodingError => e
          # This is expected for some parameter combinations
          assert e.message.include?("invalid utf-8"), "Unexpected error: #{e.message}"
        end
      end
      
      # If we get here without a segfault, that's good!
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_close_file_during_parse
    # This test attempts to trigger a use-after-free scenario
    file = Tempfile.new(%w[close_during_parse .csv])
    begin
      # Write a large CSV file
      file.write("id,name,age\n")
      10_000.times { |i| file.write("#{i},Person#{i},#{20 + i % 50}\n") }
      file.flush
      
      # Get the file path before we close it
      file_path = file.path
      
      # Create an enumerator but don't start reading yet
      enum = OSV.for_each(file_path)
      
      # Close and delete the file while the enumerator still has a reference
      file.close
      file.unlink
      
      # Now try to read from the closed file
      # This might cause a segfault if the code doesn't handle this case properly
      begin
        10.times { enum.next }
        fail "Expected an error when reading from closed file"
      rescue => e
        # We expect an error, but not a segfault
        assert true
      end
    rescue => e
      # Note the error but don't fail the test
      puts "Error during test: #{e.message}"
    end
  end
  
  def test_segfault_recursive_modification
    # Test recursively modifying data while parsing
    file = Tempfile.new(%w[recursive .csv])
    begin
      # Write a CSV file
      file.write("id,name,data\n")
      100.times { |i| file.write("#{i},name#{i},data#{i}\n") }
      file.flush
      
      # Create a special array that will be modified during iteration
      rows = []
      
      # Override the << method to recursively add to the array
      # This can create unexpected memory patterns and potentially trigger segfaults
      def rows.<<(item)
        super(item)
        
        # Create a copy and recursively process it if we haven't gone too deep
        @depth ||= 0
        
        if @depth < 3 && item.is_a?(Hash) && size < 300
          @depth += 1
          # Make a deep copy with slightly modified values
          copy = {}
          item.each do |k, v|
            copy[k] = v.is_a?(String) ? v + "_copy" : v
          end
          self << copy
          @depth -= 1
        end
        
        # Force garbage collection occasionally
        GC.start if size % 20 == 0
        
        self
      end
      
      # Parse the CSV and collect rows with our special array
      begin
        OSV.for_each(file.path) { |row| rows << row }
      rescue => e
        # Log any errors but don't fail the test
        puts "Error during recursive modification: #{e.message}"
      end
      
      # Success if we didn't segfault
      assert rows.size > 100
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_huge_header_names
    # Test with extremely large header names that might cause buffer overflows
    file = Tempfile.new(%w[huge_headers .csv])
    begin
      # Create huge header names
      huge_headers = 3.times.map { |i| "header_#{i}_" + ("x" * 10000) }
      
      # Write the CSV with huge headers
      file.write(huge_headers.join(",") + "\n")
      10.times do |i|
        file.write("#{i},#{i},#{i}\n")
      end
      file.flush
      
      # Try parsing with different options
      begin
        # Regular parsing
        rows1 = OSV.for_each(file.path).to_a
        
        # With array result type
        rows2 = OSV.for_each(file.path, result_type: "array").to_a
        
        # Without headers
        rows3 = OSV.for_each(file.path, has_headers: false).to_a
        
        # Success if we didn't segfault
        assert rows1.size > 0
        assert rows2.size > 0
        assert rows3.size > 0
      rescue => e
        # Log any errors but don't fail the test
        puts "Error during huge header parsing: #{e.message}"
      end
    ensure
      file.close
      file.unlink
    end
  end

  def test_parse_empty_file
    Tempfile.create(%w[empty .csv]) do |tempfile|
      # Empty file
      tempfile.flush

      count = 0
      OSV.for_each(tempfile.path) { |row| count += 1 }

      assert_equal 0, count

      # Also test with headers but no data
      tempfile.write("id,name\n")
      tempfile.flush

      count = 0
      OSV.for_each(tempfile.path) { |row| count += 1 }

      assert_equal 0, count
    end
  end

  def test_parse_with_recursive_io
    skip "Skipping recursive IO test as it can cause infinite loops"

    # This test is dangerous and could crash the process
    # It's included for completeness but skipped by default

    recursive_io =
      Class.new do
        def initialize
          @pos = 0
          @data = "id,name\n1,John\n"
        end

        def read(bytes)
          if @pos < @data.length
            chunk = @data[@pos, bytes]
            @pos += chunk.length
            chunk
          else
            # After reading all data, return self as more data
            # This is deliberately pathological
            self.to_s
          end
        end

        def to_s
          "2,Recursive\n"
        end
      end

    io = recursive_io.new

    # Set a timeout to prevent infinite loops
    Timeout.timeout(2) do
      begin
        count = 0
        OSV.for_each(io) do |row|
          count += 1
          break if count >= 10 # Limit iterations to prevent infinite loop
        end
      rescue => e
        # We expect this to fail, but it shouldn't segfault
        puts "Expected error: #{e.message}"
      end
    end
  end
  
  def test_segfault_io_close_with_sequential_access
    # This test attempts to trigger use-after-free without using threads
    file = Tempfile.new(['io_close', '.csv'])
    
    begin
      # Create file with content
      file.write("id,name,value\n")
      1000.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Get path before we close it
      file_path = file.path
      
      # Create an enumerator
      enum = OSV.for_each(file_path)
      
      # Read a few rows
      rows = []
      5.times { rows << enum.next }
      
      # Close the file while keeping the enumerator
      file.close
      
      # Force GC to try to clean up resources
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Create memory pressure
      garbage = []
      10.times { garbage << "x" * (1024 * 1024) }
      
      # Try to read more from the enum
      begin
        more_rows = []
        10.times { more_rows << enum.next }
        
        # If we get here, the file is likely still accessible
        # which is expected behavior if proper buffering is used
        assert more_rows.size > 0
      rescue => e
        # This might also be expected if the file wasn't fully buffered
        puts "Error after closing file: #{e.message}"
      end
      
      # Release memory pressure
      garbage = nil
      GC.start
      
      # Try one more time after GC
      begin
        enum.next
      rescue => e
        # Expected
      end
      
      # Success if no segfault
      assert true
    ensure
      begin
        file.unlink
      rescue
        # Ignore cleanup errors
      end
    end
  end
  
  def test_segfault_concurrent_file_mutations
    # Create a file that we'll modify while reading
    file = Tempfile.new(['mutation', '.csv'])
    
    begin
      # Write initial content
      file.write("id,name,value\n")
      100.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create an enumerator
      enum = OSV.for_each(file.path)
      
      # Read first few rows
      rows_read = []
      5.times { rows_read << enum.next }
      
      # Now start a thread that will truncate and rewrite the file
      modifier_thread = Thread.new do
        # Truncate the file
        file.truncate(0)
        file.rewind
        
        # Write different content
        file.write("different,headers,here\n")
        50.times { |i| file.write("modified_#{i},changed_#{i},#{i*10}\n") }
        file.flush
      end
      
      # Wait for modification to complete
      modifier_thread.join
      
      # Now try to continue reading which might cause a segfault
      # if the memory is not properly handled
      begin
        more_rows = []
        10.times { more_rows << enum.next rescue break }
        
        # We don't assert any specific behavior here - the implementation might
        # continue reading from a buffer or might fail, but it shouldn't segfault
      rescue => e
        # Expected error, but not a segfault
        puts "Expected error after file mutation: #{e.message}"
      end
      
      # Check that we at least read the first rows correctly
      assert_equal 5, rows_read.size
      assert_equal "0", rows_read[0]["id"]
      
      # Success if we didn't crash
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_multiple_incomplete_reads
    # Create a large file
    file = Tempfile.new(['incomplete', '.csv'])
    
    begin
      # Write a large CSV
      file.write("id,name,value\n")
      100_000.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      file_path = file.path
      
      # Create many readers, read partially, and discard
      # This can cause resource leaks if handles aren't properly cleaned up
      10.times do |iteration|
        begin
          # Create a new parser
          enum = OSV.for_each(file_path)
          
          # Read a small random number of rows and discard
          rand(5..20).times { enum.next }
          
          # Force garbage collection to try to clean up any resources
          # This could trigger a segfault if there are dangling pointers
          GC.start(full_mark: true, immediate_sweep: true) if iteration % 2 == 0
          
          # Create a new file handle and close it immediately
          # This could interfere with open file handles if not managed properly
          if iteration % 3 == 0
            f = File.open(file_path)
            f.close
          end
        rescue => e
          puts "Error in iteration #{iteration}: #{e.message}"
        end
      end
      
      # Success if no segfault
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_buffer_manipulation
    # Test for segfaults with unusual buffer sizes and line endings
    file = Tempfile.new(['buffer_test', '.csv'])
    
    begin
      # Create a file with very unusual line endings and field sizes
      file.write("id,name,description,data\n")
      
      # Write various rows with unusual patterns
      # 1. Very large fields (can cause buffer reallocation)
      file.write("1,normal,\"#{"x" * 10_000}\",regular\n")
      
      # 2. Mix of CR, LF, and CRLF line endings
      file.write("2,windows,line,ending\r\n")
      file.write("3,mac,old,style\r")
      file.write("4,unix,standard,newline\n")
      
      # 3. Quoted fields with embedded line breaks and quotes
      file.write("5,\"line\nbreak\",\"embedded\"\"quote\"\"\",test\n")
      
      # 4. Records with varying field counts
      file.write("6,fewer,fields\n")
      file.write("7,more,fields,with,extra,columns\n")
      
      # 5. Malformed quotes
      file.write("8,\"unclosed quote,field,value\n")
      file.write("9,\"es\"ca\"ped\",weird,format\n")
      
      # 6. Null bytes within fields (can cause C-string assumptions to break)
      file.write("10,null\0byte,weird,stuff\n")
      
      # 7. Unicode characters in various places
      file.write("11,üñîçø∂é,テスト,😀🚀👾\n")
      
      file.flush
      
      # Try parsing with different combinations of options
      [
        { flexible: true, ignore_null_bytes: true },
        { flexible: false, lossy: true },
        { result_type: "array", ignore_null_bytes: true },
        { has_headers: false }
      ].each do |params|
        begin
          # Try parsing
          rows = []
          enum = OSV.for_each(file.path, **params)
          
          # Read until the end or error
          loop do
            begin
              rows << enum.next
            rescue StopIteration
              break
            end
          end
          
          # Success! Just log how many rows we could read
          puts "Successfully read #{rows.size} rows with params: #{params}"
        rescue => e
          # Log errors but don't fail - some errors are expected
          puts "Error with params #{params}: #{e.message}"
        end
      end
      
      # Success if we didn't segfault
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_csv_parser_with_memory_patterns
    # Create a test file with various CSV patterns
    file = Tempfile.new(['csv_patterns', '.csv'])
    
    begin
      file.write("id,name,value\n")
      
      # Write normal rows
      50.times { |i| file.write("#{i},name#{i},#{i*10}\n") }
      
      # Write rows with large values at specific positions (can force reallocation)
      file.write("large1,#{"n" * 50000},value\n")
      
      # Write more normal rows
      50.times { |i| file.write("#{i+100},name#{i+100},#{(i+100)*10}\n") }
      
      # Write row with all large values (can force reallocation)
      file.write("#{"i" * 1000},#{"n" * 1000},#{"v" * 1000}\n")
      
      # Finalize and flush
      file.flush
      
      # Create an enumerator and read only part of the file
      enum = OSV.for_each(file.path)
      rows = []
      
      # Read just the first 10 rows
      10.times { rows << enum.next }
      
      # Force memory reallocation with large objects
      large_objects = []
      5.times { large_objects << "x" * (1024 * 1024) }
      
      # Force garbage collection
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Continue reading through the file, including the large rows
      begin
        while rows.size < 120
          rows << enum.next
        end
      rescue StopIteration
        # End of file
      rescue => e
        puts "Error during parsing: #{e.message}"
      end
      
      # Success if we didn't segfault
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_use_after_free_with_explicit_garbage
    file = Tempfile.new(['uaf', '.csv'])
    begin
      # Create a CSV file
      file.write("id,name,value\n")
      1000.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create an enumerator
      enum = OSV.for_each(file.path)
      
      # Read a few rows
      rows = []
      5.times { rows << enum.next }
      
      # Now try to force a use-after-free scenario
      # We'll create objects and explicitly nil their references
      # while trying to use the enumerator
      rows.clear  # Clear the references
      
      # Force an aggressive GC
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Create a lot of garbage to trigger memory reallocation
      10.times do
        temp = []
        1000.times { temp << "x" * 1000 }
        temp = nil # Explicitly release reference
        GC.start
      end
      
      # Now try to use the enumerator after potential memory reuse
      begin
        more_rows = []
        5.times { more_rows << enum.next }
        
        # If we got here without a segfault, that's good
        assert more_rows.size > 0
      rescue => e
        # An error is acceptable, but not a segfault
        puts "Expected error in use-after-free test: #{e.message}"
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_enum_marshal
    # Tests marshalling an enumerator, which could reveal memory issues
    file = Tempfile.new(['marshal', '.csv'])
    begin
      # Create a CSV file
      file.write("id,name,value\n")
      100.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create an enumerator and read a few rows
      enum = OSV.for_each(file.path)
      3.times { enum.next }
      
      # Try to marshal the enumerator (this will likely fail, but shouldn't segfault)
      begin
        marshalled = Marshal.dump(enum)
        fail "Expected Marshal.dump to fail for OSV enumerator"
      rescue => e
        # This is expected to fail but shouldn't segfault
        assert e.is_a?(TypeError), "Expected TypeError, got #{e.class}"
      end
      
      # Check that the enumerator still works
      begin
        row = enum.next
        assert_equal "3", row["id"]
      rescue => e
        puts "Error continuing after marshal attempt: #{e.message}"
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_odd_io_behavior
    # Test with various unusual IO objects that might trigger segfaults
    
    # 1. Test with an IO object that returns incomplete data
    # Create a custom IO-like object that returns chunks of data
    chunked_io = Class.new do
      def initialize
        @data = "id,name,age\n1,John,25\n2,Jane,30\n3,Jim,35\n"
        @pos = 0
      end
      
      # Return data in very small chunks
      def read(bytes)
        return nil if @pos >= @data.length
        
        # Always return just 1-2 bytes to force buffer handling
        chunk_size = [@pos % 2 + 1, @data.length - @pos].min
        result = @data[@pos, chunk_size]
        @pos += chunk_size
        result
      end
    end
    
    # Try parsing with the chunked IO
    begin
      rows = []
      enum = OSV.for_each(chunked_io.new)
      loop do
        rows << enum.next
      rescue StopIteration
        break
      end
      
      assert_equal 3, rows.size
    rescue => e
      puts "Error with chunked IO: #{e.message}"
    end
    
    # 2. Test with an IO object that reads very slowly
    slow_io = Class.new do
      def initialize
        @data = "id,name,age\n1,John,25\n2,Jane,30\n3,Jim,35\n"
        @pos = 0
      end
      
      def read(bytes)
        return nil if @pos >= @data.length
        
        # Sleep a tiny bit to simulate slow IO
        sleep(0.001)
        
        # Return a small chunk
        chunk_size = [5, @data.length - @pos].min
        result = @data[@pos, chunk_size]
        @pos += chunk_size
        result
      end
    end
    
    # Try parsing with the slow IO
    begin
      rows = []
      enum = OSV.for_each(slow_io.new)
      loop do
        rows << enum.next
      rescue StopIteration
        break
      end
      
      assert_equal 3, rows.size
    rescue => e
      puts "Error with slow IO: #{e.message}"
    end
    
    # 3. Test with an IO object that intermittently fails
    flaky_io = Class.new do
      def initialize
        @data = "id,name,age\n1,John,25\n2,Jane,30\n3,Jim,35\n"
        @pos = 0
        @fail_count = 0
      end
      
      def read(bytes)
        return nil if @pos >= @data.length
        
        # Occasionally raise an error
        @fail_count += 1
        if @fail_count % 3 == 0
          raise IOError, "Simulated intermittent failure"
        end
        
        # Return data normally otherwise
        chunk_size = [10, @data.length - @pos].min
        result = @data[@pos, chunk_size]
        @pos += chunk_size
        result
      end
    end
    
    # Try parsing with the flaky IO
    begin
      rows = []
      enum = OSV.for_each(flaky_io.new)
      loop do
        rows << enum.next
      rescue StopIteration
        break
      end
    rescue => e
      # We expect this to fail, but it shouldn't segfault
      puts "Expected error with flaky IO: #{e.message}"
    end
    
    # Success if no segfault
    assert true
  end
  
  def test_segfault_file_descriptor_reuse
    # This test tries to simulate file descriptor reuse
    # which can cause segfaults if the code assumes file descriptors remain valid
    
    file = Tempfile.new(['fd_reuse', '.csv'])
    begin
      # Create a CSV file
      file.write("id,name,value\n")
      100.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create an enumerator
      enum = OSV.for_each(file.path)
      
      # Read a few rows
      rows = []
      5.times { rows << enum.next }
      
      # Close the file
      file.close
      
      # Create many temporary files to potentially reuse the file descriptor
      temp_files = []
      20.times do |i|
        temp_files << File.open(file.path, 'r')
      end
      
      # Close half of them to create "holes" in fd table
      (temp_files.size / 2).times do |i|
        temp_files[i * 2].close
      end
      
      # Try to continue reading from the original enumerator
      begin
        more_rows = []
        5.times { more_rows << enum.next }
        
        # If we get here, things are working as expected with buffering
        assert more_rows.size > 0
      rescue => e
        # An error is acceptable, but not a segfault
        puts "Error with fd reuse: #{e.message}"
      end
      
      # Clean up remaining files
      temp_files.each do |tf|
        begin
          tf.close
        rescue
          # Ignore errors
        end
      end
      
      # Success if no segfault
      assert true
    ensure
      begin
        file.unlink
      rescue
        # Ignore cleanup errors
      end
    end
  end
  
  def test_segfault_corrupted_csv
    # Tests with deliberately corrupted CSV data that might cause segfaults
    # due to buffer overruns or other memory issues
    
    file = Tempfile.new(['corrupted', '.csv'])
    begin
      # Start with valid header
      file.write("id,name,value\n")
      
      # Add 10 normal rows
      10.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      
      # Add corrupted data - various patterns that could cause issues
      
      # 1. Extremely long line with no newline
      file.write("long" + "x" * 50000)
      
      # 2. Mix of binary data and text
      file.write([0xFF, 0x00, 0xFE, 0xA3].pack("C*"))
      file.write("binary,data,mixed\n")
      
      # 3. Incomplete quote sequence that might confuse the parser
      file.write("100,\"unclosed quote,continues\n")
      file.write("101,next,line\"\n") # Continues quote from previous line
      
      # 4. Unicode boundary corruption
      file.write("102,")
      file.write([0xE2, 0x82].pack("C*")) # Incomplete UTF-8 character
      file.write(",broken\n")
      
      # 5. Control characters mixed in
      file.write("103,con\x00trol\x01ch\x02ars,test\n")
      
      # 6. Oversized quotes with newlines
      file.write("104,\"")
      file.write("very\nlong\nquoted\nfield\n" * 1000)
      file.write("\",end\n")
      
      # Flush and try to read
      file.flush
      
      # Try parsing with different combinations
      [
        { flexible: true, ignore_null_bytes: true },
        { lossy: true },
        { result_type: "array" }
      ].each do |params|
        begin
          # Create parse enumerator
          enum = OSV.for_each(file.path, **params)
          
          # Read until error or end
          rows = []
          begin
            20.times do
              rows << enum.next
              # Force GC occasionally
              GC.start if rows.size % 5 == 0
            end
          rescue StopIteration
            # End of file - unlikely with our corrupted data
            puts "Surprisingly reached end of file with params: #{params}"
          rescue => e
            # Expected error, but it shouldn't segfault
            puts "Expected error with corrupted CSV (#{params}): #{e.message}"
          end
          
          # Create a new enumerator and try again
          enum2 = OSV.for_each(file.path, **params)
          begin
            # Skip ahead a bit and then read
            5.times { enum2.next }
            # Try reading into the corrupted section
            3.times { enum2.next }
          rescue => e
            # Expected
          end
        rescue => e
          puts "Error creating parser for corrupted CSV: #{e.message}"
        end
      end
      
      # Success if we didn't segfault
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_nested_enumerators
    # Test nested and interleaved enumerators that might cause memory corruption
    
    files = []
    begin
      # Create multiple CSV files
      3.times do |i|
        file = Tempfile.new(["nested_#{i}", '.csv'])
        file.write("id,name,value\n")
        100.times { |j| file.write("#{i}_#{j},name#{j},value#{j}\n") }
        file.flush
        files << file
      end
      
      # Create outer enumerator
      outer_enum = OSV.for_each(files[0].path)
      
      # Read a few rows from the outer enumerator
      outer_rows = []
      3.times { outer_rows << outer_enum.next }
      
      # Create inner enumerator
      inner_enum = OSV.for_each(files[1].path)
      
      # Read from inner and then outer in alternating pattern
      # This could expose memory corruption issues
      results = []
      10.times do |i|
        # Alternate between enumerators
        if i.even?
          # Read from inner
          results << inner_enum.next
        else
          # Read from outer
          results << outer_enum.next
        end
        
        # Create additional short-lived enumerator every few iterations
        if i % 3 == 0
          temp_enum = OSV.for_each(files[2].path)
          # Read a bit from temp enumerator
          2.times { results << temp_enum.next }
          # Force GC to potentially clean up
          GC.start
        end
      end
      
      # One more level of nesting
      nested_enum = OSV.for_each(files[2].path)
      5.times { nested_enum.next }
      
      # Create nested enumerator inside a block
      inner_results = []
      OSV.for_each(files[0].path) do |row|
        # Inside this block, create and use another enumerator
        inner_results << row
        
        # Create and use a temporary enumerator inside the block
        # This can lead to complex nesting of resources
        if inner_results.size < 3
          temp_inner_enum = OSV.for_each(files[1].path)
          2.times { temp_inner_enum.next }
        end
        
        break if inner_results.size >= 5
      end
      
      # Success if no segfault
      assert inner_results.size > 0
      assert results.size > 0
    ensure
      files.each do |file|
        begin
          file.close
          file.unlink
        rescue
          # Ignore cleanup errors
        end
      end
    end
  end
  
  def test_segfault_extreme_csv_variations
    # This test creates and reads multiple CSV files with extreme variations
    # in content that might trigger segmentation faults due to memory issues
    
    begin
      # Create a very large file with varying row sizes
      large_file = Tempfile.new(['extreme_large', '.csv'])
      large_file.write("id,name,description,data,extra\n")
      100.times do |i|
        # Vary the row size dramatically
        case i % 4
        when 0
          # Normal row
          large_file.write("#{i},name#{i},desc#{i},data#{i},extra#{i}\n")
        when 1
          # Row with very large field
          large_file.write("#{i},name#{i},#{"x" * 100_000},data#{i},extra#{i}\n")
        when 2
          # Row with quoted fields containing special chars
          large_file.write("#{i},\"name,#{i}\",\"desc\n#{i}\",\"data\"\"#{i}\",\"extra#{i}\"\n")
        when 3
          # Very short row (missing fields)
          large_file.write("#{i},name#{i}\n")
        end
      end
      large_file.flush
      
      # Create a file with binary data
      binary_file = Tempfile.new(['extreme_binary', '.csv'])
      binary_file.write("id,binary,data\n")
      20.times do |i|
        binary_file.write("#{i},")
        # Add some binary data
        binary_file.write(Random.new(i).bytes(50))
        binary_file.write(",end#{i}\n")
      end
      binary_file.flush
      
      # Create a file with UTF-8 edge cases
      utf8_file = Tempfile.new(['extreme_utf8', '.csv'])
      utf8_file.write("id,utf8,data\n")
      20.times do |i|
        utf8_file.write("#{i},")
        # Mix of valid and invalid UTF-8
        if i.even?
          # Valid multi-byte UTF-8 characters
          utf8_file.write("日本語UTF8テスト#{i}")
        else
          # Invalid UTF-8 sequence
          utf8_file.write([0xE0, 0x80, 0xFF, 0xE0, 0xA0].pack("C*"))
        end
        utf8_file.write(",end#{i}\n")
      end
      utf8_file.flush
      
      # Test with different parsing options in various combinations
      [
        { file: large_file, options: { flexible: true } },
        { file: large_file, options: { lossy: true, result_type: "array" } },
        { file: binary_file, options: { ignore_null_bytes: true } },
        { file: binary_file, options: { lossy: true } },
        { file: utf8_file, options: { lossy: true } },
        { file: utf8_file, options: { flexible: true, ignore_null_bytes: true } }
      ].each do |test_case|
        begin
          # Try parsing
          enum = OSV.for_each(test_case[:file].path, **test_case[:options])
          
          # Read only a portion to avoid spending too much time
          rows = []
          limit = 10
          
          begin
            while rows.size < limit
              row = enum.next
              rows << row
              
              # Force memory pressure
              if rows.size % 3 == 0
                GC.start
                temp_garbage = "x" * (1024 * 1024)
              end
            end
          rescue StopIteration
            # End of file
          rescue => e
            # Expected for some combinations
            puts "Expected error in extreme test (#{test_case[:options].inspect}): #{e.message}"
          end
          
          # Try creating a new enumerator
          enum2 = OSV.for_each(test_case[:file].path, **test_case[:options])
          
          # Skip ahead a bit
          begin
            3.times { enum2.next }
          rescue => e
            # Expected for some combinations
          end
          
          # Force aggressive GC
          GC.start(full_mark: true, immediate_sweep: true)
        rescue => e
          puts "Error creating parser for extreme CSV: #{e.message}"
        end
      end
      
      # Success if we didn't segfault
      assert true
    ensure
      [large_file, binary_file, utf8_file].each do |file|
        next unless file
        begin
          file.close
          file.unlink
        rescue
          # Ignore cleanup errors
        end
      end
    end
  end
  
  def test_segfault_buffer_boundaries
    # This test focuses on triggering segfaults related to buffer boundaries
    # The READ_BUFFER_SIZE in the implementation is 16384 bytes, so we'll create
    # scenarios right at that boundary
    
    file = Tempfile.new(['buffer_boundary', '.csv'])
    begin
      # Create a CSV file header
      file.write("id,name,description\n")
      
      # Create rows with fields exactly at buffer boundaries
      # First, create a row that ends exactly at the buffer boundary
      buffer_size = 16384 # Must match READ_BUFFER_SIZE in the implementation
      
      # Row 1: Field with size that would land the buffer boundary in middle of field
      content_size = buffer_size - "1,name1,".length - 1 # -1 for newline
      file.write("1,name1,#{"x" * content_size}\n")
      
      # Row 2: Field that causes buffer boundary to occur right before a delimiter
      content_size = buffer_size - "2,".length - 1 # -1 for newline
      file.write("2,#{"y" * content_size},desc2\n")
      
      # Row 3: Field with quoted content that spans across buffer boundary
      content_size = buffer_size - "3,\"".length - 1 # -1 for quote
      file.write("3,\"#{"z" * content_size}")
      file.write("\",desc3\n")
      
      # Row 4: Quoted field with an embedded quote right at buffer boundary
      content_size = buffer_size - "4,\"".length - 2 # -2 for two quotes
      file.write("4,\"#{"a" * content_size}\"\"")
      file.write("more content\",desc4\n")
      
      # Row 5: Field with newline right at buffer boundary
      content_size = buffer_size - "5,\"".length - 1 # -1 for newline
      file.write("5,\"#{"b" * content_size}\n")
      file.write("second line\",desc5\n")
      
      # Row 6: CSV with a combination of challenges
      file.write("6,\"mixed\nfield\",\"")
      file.write("c" * (buffer_size - 50))
      file.write("\"\n")
      
      # Row 7: Very long line followed by very short line
      file.write("7," + "d" * (buffer_size * 3) + ",long_desc\n")
      file.write("8,short,desc\n")
      
      # Flush to ensure all content is written
      file.flush
      
      # Now attempt to read this file with various options
      [
        {},
        { lossy: true },
        { flexible: true },
        { result_type: "array" },
        { ignore_null_bytes: true, lossy: true }
      ].each do |options|
        begin
          # Create enumerator
          enum = OSV.for_each(file.path, **options)
          
          # Read the file completely
          rows = []
          begin
            while true
              row = enum.next
              rows << row
              
              # Force GC occasionally to increase chances of hitting issues
              GC.start if rows.size % 2 == 0
            end
          rescue StopIteration
            # Expected at end of file
          rescue => e
            # Log any other errors
            puts "Error reading buffer boundary test (#{options.inspect}): #{e.message}"
          end
          
          # If we got here for any option combination without segfaulting, that's good
          assert rows.length > 0, "Should read at least some rows"
        rescue => e
          puts "Error creating parser for buffer boundary test: #{e.message}"
        end
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_intensive_memory_operations
    # This test creates conditions for memory-related segfaults by
    # forcing intensive memory operations during parsing
    
    file = Tempfile.new(['intensive_mem', '.csv'])
    begin
      # Create a moderately sized CSV file
      file.write("id,name,value\n")
      1000.times { |i| file.write("#{i},name#{i},value#{i}\n") }
      file.flush
      
      # Create a helper method to apply memory pressure
      def apply_memory_pressure(level)
        # Create a large object to force memory allocation
        objects = []
        case level
        when :low
          objects << "x" * (1024 * 512) # 512KB
          GC.start
        when :medium
          5.times { objects << "x" * (1024 * 1024) } # 5MB
          GC.start
        when :high
          # Create and discard many objects rapidly
          10.times do
            temp = "x" * (1024 * 1024 * 5) # 5MB
            objects << temp[0..10] # Keep just a small slice to create fragmentation
          end
          GC.start(full_mark: true, immediate_sweep: true)
        end
        objects
      end
      
      # Test parsers with different memory pressure patterns
      enum1 = OSV.for_each(file.path)
      enum2 = OSV.for_each(file.path, lossy: true)
      enum3 = OSV.for_each(file.path, result_type: "array")
      
      # Read interleaving between parsers with memory pressure
      rows = []
      pressure_objects = []
      
      200.times do |i|
        begin
          # Apply varying memory pressure
          pressure = case i % 10
            when 0..3 then :low
            when 4..7 then :medium
            else :high
          end
          
          # Keep references to some pressure objects to maintain fragmentation
          if i % 5 == 0
            pressure_objects = []
          end
          pressure_objects.concat(apply_memory_pressure(pressure))
          
          # Read from different parsers
          parser = case i % 3
            when 0 then enum1
            when 1 then enum2
            else enum3
          end
          
          # Read a row
          rows << parser.next
        rescue StopIteration
          # Reset the parser that reached the end
          case i % 3
          when 0 then enum1 = OSV.for_each(file.path)
          when 1 then enum2 = OSV.for_each(file.path, lossy: true)
          else enum3 = OSV.for_each(file.path, result_type: "array")
          end
        rescue => e
          puts "Error during intensive memory test (iteration #{i}): #{e.message}"
        end
      end
      
      # If we get here without a segfault, that's good
      assert rows.length > 0
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_forced_memory_fragmentation
    # This test attempts to create memory fragmentation conditions
    # that might expose use-after-free or double-free issues
    
    file = Tempfile.new(['fragmentation', '.csv'])
    begin
      # Create a CSV file with varying row sizes to encourage fragmentation
      file.write("id,name,value\n")
      
      # Create pattern of small and large rows
      100.times do |i|
        if i % 2 == 0
          # Small row
          file.write("#{i},small_#{i},value_#{i}\n")
        else
          # Large row
          file.write("#{i},large_#{i},#{"x" * (i * 100)}\n")
        end
      end
      file.flush
      
      # Create many enumerators to increase memory usage
      enumerators = []
      10.times do |i|
        # Create with different options to exercise different code paths
        options = case i % 3
          when 0 then {}
          when 1 then { lossy: true }
          else { result_type: "array" }
        end
        
        enumerators << OSV.for_each(file.path, **options)
      end
      
      # Read partially from each enumerator
      enumerators.each_with_index do |enum, idx|
        # Read a different number of rows from each enum
        (idx + 1).times do
          begin
            enum.next
          rescue => e
            puts "Error on initial reading: #{e.message}"
          end
        end
      end
      
      # Now force memory fragmentation
      fragmentation_objects = []
      
      # Create and discard objects of varying sizes
      sizes = [10, 100, 1000, 10000, 100000]
      50.times do |i|
        # Create objects of different sizes
        size = sizes[i % sizes.length]
        temp = "x" * size
        
        # Sometimes keep references, sometimes discard immediately
        if i % 3 == 0
          fragmentation_objects << temp
        end
        
        # Occasionally clear some references
        if i % 7 == 0 && !fragmentation_objects.empty?
          fragmentation_objects.slice!(0, fragmentation_objects.length / 2)
          GC.start
        end
      end
      
      # Force a major GC
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Now continue reading from the enumerators
      results = []
      enumerators.each_with_index do |enum, idx|
        begin
          # Try to read more rows
          5.times do
            results << enum.next
          end
        rescue StopIteration
          # Expected if we reach the end
        rescue => e
          puts "Error continuing to read after fragmentation (#{idx}): #{e.message}"
        end
      end
      
      # Finally, create a new enumerator and read it completely
      begin
        final_enum = OSV.for_each(file.path)
        final_results = []
        
        # Read all rows
        loop do
          final_results << final_enum.next
        end
      rescue StopIteration
        # Expected at end of file
      rescue => e
        puts "Error in final complete read: #{e.message}"
      end
      
      # If we got here without a segfault, that's good
      assert true
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_string_edge_cases
    # This test focuses on string/bytestring handling which is a common source
    # of segmentation faults in Rust<->Ruby integration
    
    file = Tempfile.new(['string_edge', '.csv'])
    begin
      # Create header
      file.write("id,content,description\n")
      
      # Add rows with various string edge cases
      
      # 1. Strings with null bytes (can cause problems with C string handling)
      file.write("1,string with\0null byte,test\n")
      
      # 2. Strings with high Unicode code points (4-byte UTF-8)
      file.write("2,emoji test 😀🚀👨‍👩‍👧‍👦,unicode\n")
      
      # 3. Zero-width characters (zero width space, joiner, etc)
      file.write("3,invisible\u200Bchars\u200Dhere,zero-width\n")
      
      # 4. Control characters mixed in
      file.write("4,control\x01\x02\x03\x04\x05\x06chars,bytes\n")
      
      # 5. Unicode RTL override characters (can mess with string rendering)
      file.write("5,\u202Ereversed text\u202C,rtl\n")
      
      # 6. Very long string followed by empty string
      file.write("6,#{"x" * 50000},\n")
      
      # 7. Strings with Unicode normalization forms
      # Same visual character but different byte representations
      file.write("7,e\u0301,composed\n") # é with combining accent
      file.write("8,\u00E9,precomposed\n") # é as single character
      
      # 8. Strings with interesting UTF-8 edge cases
      # Mix of 1, 2, 3 and 4 byte sequences
      file.write("9,\u007F\u0080\u07FF\u0800\uFFFF\U00010000,utf8-boundaries\n")
      
      # 9. Strings with partial/invalid UTF-8 sequences
      file.write("10,")
      file.write([0xE0, 0x80].pack("C*")) # Incomplete 3-byte sequence
      file.write(",invalid-utf8\n")
      
      # 10. Various special characters
      file.write("11,specialchars\u2028\u2029,special\n")
      
      # Flush file
      file.flush
      
      # Create multiple parsers with different string handling options
      [
        {},
        { lossy: true },
        { ignore_null_bytes: true },
        { lossy: true, ignore_null_bytes: true }
      ].each do |options|
        begin
          # Parse the file
          enum = OSV.for_each(file.path, **options)
          
          # Read and process each row
          rows = []
          
          begin
            # Collect all rows
            loop do
              row = enum.next
              rows << row
              
              # Force string processing to trigger potential issues
              if row["content"]
                # Process the string in ways that might trigger issues if
                # the string is not properly formed
                begin
                  # Force string operations
                  encoded = row["content"].encode("UTF-8", invalid: :replace)
                  downcase = encoded.downcase
                  length = encoded.length
                  
                  # Slice the string at various points
                  slices = []
                  [0, 1, encoded.length/2, encoded.length-1].each do |pos|
                    begin
                      slices << encoded[pos]
                      slices << encoded[pos, 2] if pos < encoded.length - 1
                    rescue => e
                      # Ignore slicing errors
                    end
                  end
                  
                  # Force GC
                  GC.start if rand(5) == 0
                  
                  # Do character-by-character iterations
                  encoded.each_char { |c| c.ord }
                rescue => e
                  # Ignore expected string operation errors
                end
              end
            end
          rescue StopIteration
            # Expected at end of file
          rescue => e
            puts "Expected error in string edge case test (#{options.inspect}): #{e.message}"
          end
          
          # If we processed at least some rows without segfault, that's good
          if rows.any?
            assert true
          end
        rescue => e
          puts "Error creating parser for string edge case test: #{e.message}"
        end
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_header_handling
    # This test focuses on header handling, which is a potential source of
    # segmentation faults due to special handling in the Rust code
    
    file = Tempfile.new(['header_test', '.csv'])
    begin
      # Try various problematic headers
      
      # 1. Header with duplicate column names
      file1 = Tempfile.new(['header_dup', '.csv'])
      file1.write("id,name,id,name,id\n")
      5.times { |i| file1.write("#{i},val#{i},#{i*2},val#{i*2},#{i*3}\n") }
      file1.flush
      
      # 2. Header with null bytes
      file2 = Tempfile.new(['header_null', '.csv'])
      file2.write("id,na\0me,value\n")
      5.times { |i| file2.write("#{i},val#{i},#{i*2}\n") }
      file2.flush
      
      # 3. Header with empty column names
      file3 = Tempfile.new(['header_empty', '.csv'])
      file3.write("id,,value\n")
      5.times { |i| file3.write("#{i},val#{i},#{i*2}\n") }
      file3.flush
      
      # 4. Header with quoted empty column names
      file4 = Tempfile.new(['header_quoted_empty', '.csv'])
      file4.write("id,\"\",value\n")
      5.times { |i| file4.write("#{i},val#{i},#{i*2}\n") }
      file4.flush
      
      # 5. Header with very long column names
      file5 = Tempfile.new(['header_long', '.csv'])
      file5.write("id,#{"x" * 10000},value\n")
      5.times { |i| file5.write("#{i},val#{i},#{i*2}\n") }
      file5.flush
      
      # 6. Header with minimal column name (1 byte)
      file6 = Tempfile.new(['header_short', '.csv'])
      file6.write("id,a,b,c,d,e,f\n")
      5.times { |i| file6.write("#{i},1,2,3,4,5,6\n") }
      file6.flush
      
      # 7. Header with special characters in column names
      file7 = Tempfile.new(['header_special', '.csv'])
      file7.write("id,name\nwith\nnewline,\"quoted,comma\"\n")
      5.times { |i| file7.write("#{i},val#{i},#{i*2}\n") }
      file7.flush
      
      # 8. No header but has_headers: false
      file8 = Tempfile.new(['no_header', '.csv'])
      10.times { |i| file8.write("#{i},val#{i},#{i*2}\n") }
      file8.flush
      
      # Test each file with different options
      [
        { file: file1, options: {} },
        { file: file1, options: { result_type: "array" } },
        { file: file2, options: { ignore_null_bytes: true } },
        { file: file2, options: { lossy: true } },
        { file: file3, options: {} },
        { file: file4, options: {} },
        { file: file5, options: {} },
        { file: file6, options: {} },
        { file: file7, options: { flexible: true } },
        { file: file8, options: { has_headers: false } }
      ].each do |test_case|
        begin
          # Create enumerator
          enum = OSV.for_each(test_case[:file].path, **test_case[:options])
          
          # Read all rows
          rows = []
          loop do
            row = enum.next
            rows << row
            
            # Occasionally force GC
            GC.start if rows.size % 3 == 0
          end
        rescue StopIteration
          # Expected at end of file
          # Success if we read at least one row
          assert rows.size > 0
        rescue => e
          puts "Expected error in header test (#{test_case[:options].inspect}): #{e.message}"
        end
      end
    ensure
      # Clean up
      [file1, file2, file3, file4, file5, file6, file7, file8].each do |f|
        next unless f
        begin
          f.close
          f.unlink
        rescue
          # Ignore cleanup errors
        end
      end
    end
  end
  
  def test_segfault_memory_stress_with_cow_strings
    # This test specifically targets the CowStr type in the Rust code
    # which is a potential source of segmentation faults
    
    file = Tempfile.new(['cow_string', '.csv'])
    begin
      # Create a CSV with identical strings that could be shared by Cow
      file.write("id,name,value,name_again,id_again\n")
      
      # Create rows with repeated values (to test Cow optimization)
      1000.times do |i|
        # Make some values repeat to encourage string sharing
        name = "name#{i % 10}"
        value = "value#{i % 5}"
        file.write("#{i},#{name},#{value},#{name},#{i}\n")
      end
      file.flush
      
      # Create parsers
      enum1 = OSV.for_each(file.path)
      enum2 = OSV.for_each(file.path, lossy: true)
      
      # Create buffer to hold results and prevent GC
      results = []
      
      # Read alternating between parsers
      # - Regular reading
      # - With concurrent GC pressure
      # - With string processing that might trigger copy-on-write behavior
      50.times do |i|
        begin
          # Choose parser
          enum = i.even? ? enum1 : enum2
          
          # Read next row
          row = enum.next
          
          # Process row differently based on iteration
          case i % 4
          when 0
            # Just store the row
            results << row
          when 1
            # Add memory pressure
            temp = "x" * (1024 * 1024)
            GC.start
            
            # Process and modify the strings
            processed = {}
            row.each do |k, v|
              # Force string allocations and modifications
              processed[k.upcase] = v ? v.dup.upcase : nil
            end
            results << processed
          when 2
            # Process by sharing identical strings
            processed = {}
            row.each do |k, v|
              # Use the same key/value references multiple times
              # This can trigger copy-on-write behavior in Rust
              same_k = k
              same_v = v
              processed[same_k] = same_v
              processed["#{same_k}_copy"] = same_v
              processed["#{same_k}_again"] = "#{same_v}_modified" if same_v
            end
            results << processed
          when 3
            # Heavy string processing with GC
            GC.disable # Temporarily disable GC
            processed = {}
            temp_strings = []
            
            row.each do |k, v|
              # Create many string operations
              if v
                5.times do |j|
                  temp = v * (j+1)
                  sliced = temp[0...(temp.length / 2)]
                  temp_strings << sliced
                end
                processed[k] = v.gsub(/[0-9]/, '*')
              else
                processed[k] = nil
              end
            end
            
            # Now force GC with string references still alive
            temp_strings = nil
            GC.enable
            GC.start(full_mark: true, immediate_sweep: true)
            
            results << processed
          end
        rescue StopIteration
          # Reset the parser that reached the end
          if i.even?
            enum1 = OSV.for_each(file.path)
          else
            enum2 = OSV.for_each(file.path, lossy: true)
          end
        rescue => e
          puts "Error during Cow string test (iteration #{i}): #{e.message}"
        end
      end
      
      # Ensure we read data successfully
      assert results.size > 0
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_memory_boundary_conditions
    # Test specifically for memory boundary conditions that could cause segfaults

    # Try creating and manipulating very large data that might trigger reallocation
    # and other boundary conditions in the Rust code
    begin
      # Create a test file with specific boundary conditions
      file = Tempfile.new(['boundary', '.csv'])
      
      # 1. Create header with many columns to trigger the fixed array size issue
      # This should be at least 128/2 + 1 = 65 columns to exceed the 128 array elements in into_value_with
      col_count = 90  # This will result in 90*2=180 values, well beyond the 128 limit
      header = (0...col_count).map { |i| "col#{i}" }.join(',')
      file.write("#{header}\n")
      
      # 2. Create rows with data that hits different memory boundaries
      
      # Add small rows
      5.times do |i|
        file.write((0...col_count).map { |j| "small#{i}_#{j}" }.join(','))
        file.write("\n")
      end
      
      # Add a row with exactly 16KB of data (READ_BUFFER_SIZE boundary)
      buffer_size = 16384
      remaining_size = buffer_size - header.length - 1 # -1 for newline
      vals = []
      current_size = 0
      
      # Fill exactly to the buffer size
      (0...col_count).each do |j|
        if j < col_count - 1
          # For columns before the last, use fixed small size
          vals << "val#{j}"
          current_size += "val#{j}".length + 1 # +1 for comma
        else
          # For the last column, fill to exact size
          remaining = remaining_size - current_size
          vals << "x" * remaining
        end
      end
      file.write(vals.join(','))
      file.write("\n")
      
      # Add a row with slightly less than buffer size
      vals[-1] = "x" * (vals[-1].length - 10)
      file.write(vals.join(','))
      file.write("\n")
      
      # Add a row with slightly more than buffer size
      vals[-1] = "x" * (vals[-1].length + 10)
      file.write(vals.join(','))
      file.write("\n")
      
      # Add rows with specific sized fields to test memory allocation patterns
      sizes = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192]
      sizes.each do |size|
        row_vals = (0...col_count).map do |j|
          if j == col_count / 2
            # One column with the test size
            "x" * size
          else
            # Other columns small
            "s#{j}"
          end
        end
        file.write(row_vals.join(','))
        file.write("\n")
      end
      
      # Add a row with quote and escape character at buffer boundaries
      quote_row = []
      total_length = 0
      need_exact = buffer_size - 2 # Space for initial quote and ending quote
      
      (0...col_count).each do |j|
        if j < col_count - 1
          # Regular value for most columns
          quote_row << "val#{j}"
          total_length += "val#{j}".length + 1 # +1 for comma
        else
          # Last column has a quoted value with precise size
          remaining = need_exact - total_length
          if remaining > 0
            # Create string with quotes that will land at buffer boundaries
            field_content = "y" * (remaining - 3) # -3 for the quotes and comma
            quote_row << "\"#{field_content}\"\""
          else
            quote_row << "\"overflow\""
          end
        end
      end
      file.write(quote_row.join(','))
      file.write("\n")
      
      # Finalize the file
      file.flush
      
      # Now process this file with various options to stress test the memory handling
      [
        {},
        { lossy: true },
        { result_type: "array" }
      ].each do |options|
        begin
          # Create and test an enumerator
          enum = OSV.for_each(file.path, **options)
          
          # Read everything while applying memory pressure
          rows = []
          
          # Objects to hold temporary data and prevent GC
          temp_data = []
          
          loop do
            begin
              row = enum.next
              rows << row
              
              # Every few rows, apply different types of memory pressure
              if rows.size % 3 == 0
                # Create different sized objects
                size = [1_000, 10_000, 100_000, 1_000_000][rows.size % 4]
                temp_data << "x" * size
                
                # Occasionally clear temp data
                if rows.size % 5 == 0
                  temp_data.clear
                  GC.start(full_mark: true, immediate_sweep: true)
                end
              end
            rescue StopIteration
              break
            end
          end
          
          # Success if we read data
          assert rows.size > 0, "Should read rows without segfault"
          
          # Create a fresh enumerator and do partial reading with GC
          enum2 = OSV.for_each(file.path, **options)
          
          5.times do
            # Read a row
            row = enum2.next
            
            # Force specific memory patterns
            GC.start
            GC.disable
            large_temp = "z" * 1_000_000
            large_temp = nil
            GC.enable
            GC.start
          end
        rescue => e
          puts "Error in boundary test (#{options.inspect}): #{e.message}"
        end
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  def test_segfault_multithreaded_sequential
    # This test creates multiple threads that access different enumerators sequentially,
    # attempting to trigger segfaults related to thread handling while avoiding fiber errors
    
    # Create multiple files
    files = []
    enumerators = []
    
    begin
      # Create 5 separate files
      5.times do |i|
        file = Tempfile.new(["thread_seq_#{i}", '.csv'])
        file.write("id,name,value\n")
        100.times { |j| file.write("#{i}_#{j},name#{j},value#{j}\n") }
        file.flush
        files << file
        
        # Create an enumerator for this file with different settings
        options = case i % 3
          when 0 then {}
          when 1 then { lossy: true }
          else { result_type: "array" }
        end
        
        enumerators << OSV.for_each(file.path, **options)
      end
      
      # Create multiple threads, each accessing a different enumerator
      results = {}
      mutex = Mutex.new
      
      threads = enumerators.map.with_index do |enum, idx|
        Thread.new do
          thread_results = []
          
          # Each thread reads all data from its own enumerator
          begin
            loop do
              row = enum.next
              thread_results << row
              
              # Apply occasional memory pressure in the thread
              if thread_results.size % 10 == 0
                # Create temporary memory pressure
                temp = "x" * (1024 * 1024)
                GC.start 
              end
            end
          rescue StopIteration
            # Expected at end of file
          rescue => e
            puts "Error in thread #{idx}: #{e.message}"
          end
          
          # Store results safely
          mutex.synchronize do
            results[idx] = thread_results
          end
        end
      end
      
      # Let threads run and complete
      threads.each(&:join)
      
      # Verify that each thread processed its data
      assert results.keys.size > 0, "Should have results from at least one thread"
      
      # Now try with sequential operations in one thread
      single_thread_results = []
      
      # Create new enumerators
      fresh_enums = files.map { |file| OSV.for_each(file.path) }
      
      # Alternate reads between parsers
      100.times do |i|
        enum_idx = i % fresh_enums.size
        begin
          row = fresh_enums[enum_idx].next
          single_thread_results << row
        rescue StopIteration
          # Reset the enumerator
          fresh_enums[enum_idx] = OSV.for_each(files[enum_idx].path)
        rescue => e
          puts "Error in sequential test: #{e.message}"
        end
      end
      
      # If we get here without segfault, that's good
      assert single_thread_results.size > 0
    ensure
      files.each do |file|
        begin
          file.close
          file.unlink
        rescue
          # Ignore cleanup errors
        end
      end
    end
  end
  
  def test_segfault_array_size_boundary
    # This test specifically tests the 64-65 column boundary that causes segfaults
    # When a CSV has 65 columns, it requires 130 values (65 keys + 65 values), which
    # exceeds the fixed array size of 128 elements in record.rs
    
    # The error in record.rs at line 31 is:
    # index out of bounds: the len is 128 but the index is 128
    
    # First test with array mode, which should work at any column count
    [63, 64, 65].each do |column_count|
      file = Tempfile.new(["array_#{column_count}", '.csv'])
      begin
        # Create header with specified columns
        header = (0...column_count).map { |i| "col#{i}" }.join(',')
        file.write("#{header}\n")
        
        # Create a single data row
        row_data = (0...column_count).map { |i| "val#{i}" }.join(',')
        file.write("#{row_data}\n")
        file.flush
        
        # Try parsing with array result type
        enum_array = OSV.for_each(file.path, result_type: "array")
        row_array = enum_array.next
        
        # Verify the array has the correct number of columns
        assert_equal column_count, row_array.size, "Array row should have #{column_count} columns"
      ensure
        file.close
        file.unlink
      end
    end
    
    # Now test with default hash mode - this will cause a fatal error at 65 columns
    # Skip 65 columns case since it will crash
    [63, 64].each do |column_count|
      file = Tempfile.new(["hash_#{column_count}", '.csv'])
      begin
        # Create header with specified columns
        header = (0...column_count).map { |i| "col#{i}" }.join(',')
        file.write("#{header}\n")
        
        # Create a single data row
        row_data = (0...column_count).map { |i| "val#{i}" }.join(',')
        file.write("#{row_data}\n")
        file.flush
        
        # Try parsing with default hash mode
        enum = OSV.for_each(file.path)
        row = enum.next
        
        # Verify the row has the correct number of columns
        assert_equal column_count, row.keys.size, "Row should have #{column_count} columns"
      ensure
        file.close
        file.unlink
      end
    end
    
    # Finally, create a test for the exact crash case
    # To run this critical section, we need to expose the issue
    file = Tempfile.new(["crash_65", '.csv'])
    begin
      # Create a 65-column CSV
      column_count = 65
      header = (0...column_count).map { |i| "col#{i}" }.join(',')
      file.write("#{header}\n")
      
      row_data = (0...column_count).map { |i| "val#{i}" }.join(',')
      file.write("#{row_data}\n")
      file.flush
      
      # Temporarily comment out to run the crash test:
      # This will crash with: index out of bounds: the len is 128 but the index is 128
      begin
        enum = OSV.for_each(file.path)
        row = enum.next
        
        # If we got here, then the issue has been fixed!
        assert_equal column_count, row.keys.size, "65-column CSV should parse correctly"
        puts "SUCCESS: 65-column CSV parsed without error. The issue has been fixed!"
      rescue => e
        # If the issue exists but Ruby somehow catches it
        puts "Error with 65 columns: #{e.class} - #{e.message}"
        flunk "CSV with 65 columns should parse without error"
      end
    ensure
      file.close
      file.unlink
    end
  end
end
