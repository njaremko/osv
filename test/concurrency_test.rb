# frozen_string_literal: true

require "osv"
require "minitest/autorun"

# Tests focused on concurrency and thread-safety
class ConcurrencyTest < Minitest::Test
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
end