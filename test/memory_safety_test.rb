# frozen_string_literal: true

require "osv"
require "zlib"
require "minitest/autorun"

class MemorySafetyTest < Minitest::Test
  # Test to target potential issues with Ruby string slices in RubyReader
  # Focuses on the RubyReader::String variant that uses as_slice()
  def test_string_slice_gc_safety
    begin
      # Create a large string with CSV content
      csv_string = "id,name,description\n"
      1000.times do |i|
        csv_string += "#{i},name#{i},desc#{i}\n"
      end
      
      # Create a StringIO with the string
      string_io = StringIO.new(csv_string)
      
      # Start parsing
      enum = OSV.for_each(string_io)
      
      # Read a few rows
      rows = []
      5.times { rows << enum.next }
      
      # Clear the original string reference and force GC
      csv_string = nil
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Create memory pressure by allocating large objects
      large_objects = []
      10.times { large_objects << "x" * (1024 * 1024) }
      
      # Continue reading - this would segfault if RubyReader keeps unsafe references
      # to the original string after GC
      begin
        20.times { rows << enum.next }
      rescue StopIteration
        # Expected at end of file
      end
      
      # Verify we read the expected data
      assert rows.size > 5
      assert_equal "5", rows[5]["id"] if rows.size > 5
    end
  end

  # Test for potential issues with IO object and its premature garbage collection
  def test_io_object_gc_safety
    begin
      # Create a custom IO-like object that we can control
      custom_io = Class.new do
        def initialize(data)
          @data = data
          @position = 0
        end
        
        def read(bytes)
          return nil if @position >= @data.length
          chunk = @data[@position, [bytes, 100].min] # Read in small chunks
          @position += chunk.length
          chunk
        end
      end
      
      # Create CSV data
      csv_data = "id,name,value\n"
      100.times { |i| csv_data += "#{i},name#{i},value#{i}\n" }
      
      # Create our custom IO object
      io_obj = custom_io.new(csv_data)
      
      # Create an enumerator using the custom IO
      enum = OSV.for_each(io_obj)
      
      # Read a few rows
      rows = []
      5.times { rows << enum.next }
      
      # Release references to the IO object and force GC
      io_obj = nil
      GC.start(full_mark: true, immediate_sweep: true)
      
      # Allocate objects to increase memory pressure
      10.times { "x" * (1024 * 1024) }
      
      # Try to continue reading after GC (should either work correctly or raise
      # a Ruby exception, but shouldn't segfault)
      begin
        10.times { rows << enum.next }
      rescue => e
        # If Rust has unsafe references to Ruby objects that were GC'd,
        # this might segfault instead of a proper Ruby exception
        assert_match(/io|read|closed/i, e.message)
      end
      
      # Success if we got this far without segfault
      assert true
    end
  end
  
  # Test to exercise thread safety issues with unchecked Ruby VM access
  def test_thread_safety_ruby_vm_access
    # Create a CSV file to read
    file = Tempfile.new(['thread_safety', '.csv'])
    begin
      # Create a large CSV file
      file.write("id,name,description,value,extra\n")
      500.times { |i| file.write("#{i},name#{i},desc#{i},value#{i},extra#{i}\n") }
      file.flush
      
      # Create shared data structures
      results = Queue.new
      error_count = 0
      mutex = Mutex.new
      
      # Create multiple threads that will read from the same file concurrently
      # This can expose thread safety issues with Ruby VM access
      threads = 8.times.map do |thread_id|
        Thread.new do
          begin
            # Each thread creates its own enumerator
            enum = OSV.for_each(file.path)
            
            # Skip to a different starting point
            skip_count = thread_id * 10
            skip_count.times { enum.next rescue nil }
            
            # Read rows with aggressive GC in between
            10.times do |i|
              begin
                row = enum.next
                results << [thread_id, row["id"]]
                
                # Force GC frequently
                GC.start if i % 2 == 0
                
                # Create temporary objects to increase memory pressure
                temp = "x" * (1024 * (thread_id + 1))
                temp = nil
              rescue StopIteration
                break
              rescue => e
                mutex.synchronize { error_count += 1 }
                puts "Thread #{thread_id} error: #{e.message}"
                break
              end
            end
          rescue => e
            mutex.synchronize { error_count += 1 }
            puts "Thread #{thread_id} outer error: #{e.message}"
          end
        end
      end
      
      # Wait for all threads to complete
      threads.each(&:join)
      
      # Check that we got results without segfaults
      assert results.size > 0
      assert_equal 0, error_count, "Expected no errors during concurrent parsing"
    ensure
      file.close
      file.unlink
    end
  end
  
  # Test buffer boundary handling which can cause issues with memory safety
  def test_buffer_boundary_handling
    # Create a file with content designed to test buffer boundaries
    file = Tempfile.new(['buffer_boundary', '.csv'])
    begin
      # The READ_BUFFER_SIZE in the implementation is 16384 bytes
      buffer_size = 16384
      
      # Write CSV header
      file.write("id,name,description\n")
      
      # Row 1: Create a field that ends exactly at buffer boundary
      content_size = buffer_size - "1,name1,".length - 1 # -1 for newline
      file.write("1,name1,#{"x" * content_size}\n")
      
      # Row 2: Field that causes buffer boundary to occur right before a delimiter
      content_size = buffer_size - "2,".length - 1 # -1 for newline
      file.write("2,#{"y" * content_size},desc2\n")
      
      # Row 3: Field with quoted content that spans across buffer boundary
      content_size = buffer_size - 10
      file.write("3,\"#{"z" * content_size}\",desc3\n")
      
      # Row 4: Multiple quoted fields with escaped quotes near buffer boundary
      file.write("4,\"#{"a" * (buffer_size/2 - 10)}\"\"#{"b" * 10}\",\"multi\"\"quote\"\n")
      
      # Flush to ensure content is written
      file.flush
      
      # Try parsing with different options
      [
        {},
        { result_type: "array" },
        { flexible: true },
        { lossy: true }
      ].each do |opts|
        begin
          # Parse with each option set
          enum = OSV.for_each(file.path, **opts)
          
          # Read rows while doing aggressive GC
          rows = []
          begin
            loop do
              rows << enum.next
              GC.start if rows.size % 2 == 0
            end
          rescue StopIteration
            # Expected at end of file
          end
          
          # Verify we read all rows
          assert_equal 4, rows.size, "Should have read 4 rows with options: #{opts}"
        rescue => e
          puts "Error parsing with buffer boundary test (#{opts.inspect}): #{e.message}"
          raise
        end
      end
    ensure
      file.close
      file.unlink
    end
  end
  
  # Test with different Ruby string encodings to find encoding-related memory issues
  def test_string_encoding_safety
    begin
      # Create strings with different encodings
      utf8_string = "id,name,description\n1,John,Regular\n2,José,Café\n3,你好,世界\n"
      ascii_string = utf8_string.encode("ASCII-8BIT", invalid: :replace, undef: :replace)
      utf16_string = utf8_string.encode("UTF-16LE")
      
      # Test with each encoding
      [utf8_string, ascii_string, utf16_string].each do |str|
        string_io = StringIO.new(str)
        
        begin
          # Parse the string
          enum = OSV.for_each(string_io)
          
          # Read while forcing GC
          rows = []
          begin
            while rows.size < 10
              row = enum.next
              rows << row
              GC.start if rows.size % 2 == 0
            end
          rescue StopIteration
            # Expected at end
          rescue => e
            # Some encodings may cause valid errors
            if str == utf16_string
              assert_match(/invalid|encoding/i, e.message)
            else
              raise
            end
          end
        rescue => e
          # Only UTF-16 is expected to have encoding issues
          if str == utf16_string
            assert_match(/invalid|encoding/i, e.message)
          else
            raise
          end
        end
      end
    end
  end
end