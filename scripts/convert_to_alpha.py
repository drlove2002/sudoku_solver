#!/usr/bin/env python3
import sys
import os

def convert(puzzle_str):
    mapping = {
        '1': 'A', '2': 'B', '3': 'C', '4': 'D', '5': 'E', 
        '6': 'F', '7': 'G', '8': 'H', '9': 'I', 
        'A': 'J', 'B': 'K', 'C': 'L', 'D': 'M', 'E': 'N', 
        'F': 'O', 'G': 'P', 'H': 'Q', 'I': 'R', 'J': 'S', 
        'K': 'T', 'L': 'U', 'M': 'V', 'N': 'W', 'O': 'X', 'P': 'Y',
        '.': '.', '0': '.'
    }
    
    res = ""
    for c in puzzle_str.upper():
        if c in mapping:
            res += mapping[c]
        elif c.isspace():
            continue
        else:
            res += c
    return res

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 convert_to_alpha.py <puzzle_string_or_file>")
        sys.exit(1)
    
    input_val = sys.argv[1]
    if os.path.exists(input_val):
        with open(input_val, 'r') as f:
            content = f.read()
    else:
        content = input_val
    
    # split by lines and convert
    for line in content.strip().split('\n'):
        if not line.strip(): continue
        print(convert(line))
