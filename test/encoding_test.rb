# frozen_string_literal: true

require "osv"
require "minitest/autorun"

# Tests focused on encoding handling
class EncodingTest < Minitest::Test
  def test_parse_csv_with_invalid_utf8
    invalid_utf8 = StringIO.new("id,name\n1,\xFF\xFF\n")
    assert_raises(EncodingError) do
      OSV.for_each(invalid_utf8) { |_row| }
    rescue => e
      assert e.message.include?("invalid utf-8")
      raise
    end
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
end