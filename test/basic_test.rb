# frozen_string_literal: true

require "osv"

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
end
