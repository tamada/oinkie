import sys
import csv
import argparse

def parse_arguments():
    """Parse command line arguments for the CSV fixer."""
    parser = argparse.ArgumentParser(
        description="Fix CSV files while preserving 4-line headers."
    )
    parser.add_argument("input_file", help="Path to the input CSV file")
    parser.add_argument(
        "--data-cols", 
        type=int, 
        help="Number of data columns (auto-calculated from 5th line if omitted)"
    )
    return parser.parse_args()

def get_data_col_count(file_path, manual_count):
    """Determine the data column count from the 5th line (first data row)."""
    if manual_count is not None:
        return manual_count
    with open(file_path, 'r', newline='', encoding='utf-8') as f:
        # Skip 4 header lines to find the first data row
        for _ in range(4):
            f.readline()
        first_data_line = f.readline().strip()
        if not first_data_line:
            return 0
        return len(first_data_line.split(',')) - 2

def process_row(parts, data_cols):
    """Split data parts into [ID, FullName, ...Data] with quoting."""
    if len(parts) <= data_cols + 1:
        return None
    row_id = parts[0]
    data_fields = parts[-data_cols:]
    name_parts = parts[1:-data_cols]
    full_name = ",".join(name_parts)
    return [row_id, full_name] + data_fields

def run_fixer(args, data_cols):
    """Pass headers through and fix data rows."""
    try:
        with open(args.input_file, 'r', newline='', encoding='utf-8') as f:
            # 1. Output the first 4 header lines exactly as they are
            for _ in range(4):
                sys.stdout.write(f.readline())
            
            # 2. Process remaining data rows
            writer = csv.writer(sys.stdout, quoting=csv.QUOTE_MINIMAL)
            for line in f:
                line = line.strip()
                if not line:
                    continue
                row = process_row(line.split(','), data_cols)
                if row:
                    writer.writerow(row)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)

if __name__ == "__main__":
    args = parse_arguments()
    cols = get_data_col_count(args.input_file, args.data_cols)
    run_fixer(args, cols)
