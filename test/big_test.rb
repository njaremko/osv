# frozen_string_literal: true

require "osv"
require "zlib"
require "minitest/autorun"

class BigTest < Minitest::Test
  def test_parse_csv_with_many_rows
    # Generate test data with 2000 rows
    Tempfile.create(%w[test_many_rows .csv]) do |test_file|
      test_file.write "id,name,age\n"
      2000.times { |i| test_file.write "#{i},Person#{i},#{20 + i % 50}\n" }
      test_file.close

      # Parse and verify
      actual = []
      OSV.for_each(test_file.path) { |row| actual << row }

      assert_equal 2000, actual.size
    end
  end

  def test_parse_csv_with_many_rows_stringio
    # Generate test data with 2000 rows
    io = StringIO.new
    io.write "id,name,age\n"
    2000.times { |i| io.write "#{i},Person#{i},#{20 + i % 50}\n" }
    io.rewind

    # Parse and verify
    actual = []
    OSV.for_each(io) { |row| actual << row }

    assert_equal 2000, actual.size
  end
end
