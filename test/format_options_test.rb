# frozen_string_literal: true

require "osv"
require "minitest/autorun"

# Tests focused on parsing options and formatting
class FormatOptionsTest < Minitest::Test
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
end