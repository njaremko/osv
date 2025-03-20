# frozen_string_literal: true

require "osv"
require "zlib"
require "minitest/autorun"

# Tests focused on IO handling capabilities
class IoHandlingTest < Minitest::Test
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
end