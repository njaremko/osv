# frozen_string_literal: true

require "osv"
require "minitest/autorun"

# Tests focused on stress-testing and edge cases
class StressTest < Minitest::Test
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

  def test_parse_csv_with_large_data
    skip "Skipping large data test in normal test runs" unless ENV["RUN_LARGE_TESTS"]

    # Only run during specific stress test sessions
    # Create a large file
    Tempfile.create(%w[large_data .csv]) do |tempfile|
      tempfile.write("id,name,value\n")

      # Write about 1GB of data
      100_000.times do |i|
        # ~10KB per line × 100K = ~1GB
        value = "value_#{i}_" + ("x" * 10_000)
        tempfile.write("#{i},name#{i},#{value}\n")
      end
      tempfile.flush

      # Parse the file
      count = 0
      OSV.for_each(tempfile.path) do |row|
        count += 1
        # Verify some values to ensure proper parsing
        assert_equal count - 1, row["id"].to_i
        assert_equal "name#{count - 1}", row["name"]
        assert row["value"].start_with?("value_#{count - 1}_")
        
        # Only read a portion to keep test runtime reasonable
        break if count >= 10_000
      end

      assert count > 0, "Should have read some rows"
    end
  end
end