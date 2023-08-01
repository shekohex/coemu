# Converts the Input text file to a CSV file
# Input: Text file
# Output: CSV file

# Example Input:
# 601 OfflineTG 601 0074 0 65 55 601 4294967295
# Output:
# 601,OfflineTG,601,0074,0,65,55,601,4294967295

import argparse
import csv

parser = argparse.ArgumentParser(
    prog="TextToCSV",
    description="Converts the Input text file to a CSV file",
    epilog="Example: python txt-to-csv.py --input input.txt --output output.csv",
)
parser.add_argument("-i", "--input", help="Input text file", type=str, required=True)
parser.add_argument("-o", "--output", help="Output CSV file", type=str, required=True)
args = parser.parse_args()

input_file = args.input
output_file = args.output


# Open input text file for reading
with open(input_file, "r") as f:
    lines = f.readlines()
    # Filter out empty lines
    lines = list(filter(lambda x: x.strip(), lines))

# Open output CSV file for writing
with open(output_file, "w") as csvfile:

    # Create CSV writer
    writer = csv.writer(csvfile, dialect=csv.unix_dialect, quoting=csv.QUOTE_NONE)

    # Loop through lines and write to CSV
    for line in lines:
        row = line.strip().split(" ")
        if len(row) > 0:
          writer.writerow(row)
