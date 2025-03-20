# frozen_string_literal: true

require "osv"
require "zlib"
require "minitest/autorun"

# Core functionality tests for the OSV CSV parser
class CoreFunctionalityTest < Minitest::Test
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
end