# Given the following csv file with structure:
# UniqId,Name,Type,Look,MapId,X,Y,Base,Sort,Level,Life,Defence,MagicDef
# 1,Storekeeper,0001,10,1002,0415,0351,0000,0000,0000,0000,0000,0000
# 
# This script will generate SQL statements to insert the NPCs into the database.
# One Input also is the Maps.csv file to get the map_id or to check if the map_id exists.
#
# Usage: python npcs.py --npcs npcs.csv --maps maps.csv --output npcs.sql
# Example: python npcs.py --npcs npcs.csv --maps maps.csv --output npcs.sql

import argparse
import csv

parser = argparse.ArgumentParser(description='Generate SQL statements to insert NPCs into the database.')
parser.add_argument('--npcs', dest='csv_file', help='CSV file with the NPCs')
parser.add_argument('--maps', dest='maps_file', help='CSV file with the Maps')
parser.add_argument('--output', dest='output_file', help='Output file with the SQL statements')
args = parser.parse_args()

# Read the maps file to get the map_id
maps = {}
with open(args.maps_file, 'r') as csvfile:
    reader = csv.reader(csvfile, delimiter=',', skipinitialspace=True, dialect=csv.unix_dialect)
    next(reader) # skip header
    for row in reader:
        map_id = row[0]
        maps[map_id] = True

statms = []
with open(args.csv_file, 'r') as csvfile:
    reader = csv.reader(csvfile, delimiter=',', skipinitialspace=True, dialect=csv.unix_dialect)
    next(reader) # skip header
    for row in reader:
        # We are using SQLITE so we don't need to specify the columns
        uniq_id = row[0]
        name = row[1]
        npc_type = row[2]
        look = row[3]
        map_id = row[4]
        x = row[5]
        y = row[6]
        base = row[7]
        sort = row[8]
        level = row[9]
        life = row[10]
        defence = row[11]
        magic_def = row[12]
        smtm = f"INSERT INTO npcs VALUES ({uniq_id}, '{name}', {npc_type}, {look}, {map_id}, {x}, {y}, {base}, {sort}, {level}, {life}, {defence}, {magic_def});"
        if map_id in maps:
            statms.append(smtm)
        else:
            # Add a comment to the SQL statement
            statms.append(f"-- {smtm}")

# Write the SQL statements to the output file
with open(args.output_file, 'w') as f:
    for smtm in statms:
        f.write(smtm + "\n")
