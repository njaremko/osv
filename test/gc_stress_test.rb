# frozen_string_literal: true

require "osv"
require "minitest/autorun"
require "stringio"
require "tempfile"

class GCStressTest < Minitest::Test
  def setup
    # Create a CSV string to test with
    csv = String.new("id,header1,header2\n")
    100.times do |i|
      csv << "#{i},value_#{i}_1,value_#{i}_2\n"
    end
    @csv_string = csv
    
    # Set GC to maximum stress level
    GC.stress = true
  end
  
  def teardown
    # Reset GC settings
    GC.stress = false
  end
  
  def test_parse_with_gc_stress
    # Parse the CSV with GC stress enabled
    results = []
    
    # Use a StringIO to avoid filesystem operations
    io = StringIO.new(@csv_string)
    
    # Parse with OSV
    enum = OSV.for_each(io)
    
    # Read all rows with aggressive GC between each
    count = 0
    begin
      while count < 100
        row = enum.next
        results << row
        count += 1
        
        # Force garbage collection
        GC.start(full_mark: true, immediate_sweep: true)
      end
    rescue StopIteration
      # Expected at end of file
    end
    
    # Verify we read everything
    assert_equal 100, results.size
    
    # Verify some random values
    assert_equal "0", results[0]["id"]
    assert_equal "value_0_1", results[0]["header1"]
    assert_equal "50", results[50]["id"]
    assert_equal "value_50_1", results[50]["header1"]
    assert_equal "99", results[99]["id"]
    assert_equal "value_99_2", results[99]["header2"]
  end
  
  def test_file_handle_gc_safety
    # Test with file handles that might be garbage collected
    file = Tempfile.new(['gc_stress', '.csv'])
    begin
      # Write CSV data
      file.write(@csv_string)
      file.flush
      
      # Create parser from the file
      path = file.path
      enum = OSV.for_each(path)
      
      # Read some rows with GC pressure
      10.times do
        row = enum.next
        assert_equal row["header1"], "value_#{row["id"]}_1"
        
        # Force GC
        GC.start(full_mark: true, immediate_sweep: true)
      end
    ensure
      file.close
      file.unlink
    end
  end
end