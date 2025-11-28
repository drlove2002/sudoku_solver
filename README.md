# Sudoku Solver

## Overview

This project is a Sudoku solver that represents the board with bitmasks and explores solutions at the 3×3 minigrid level. Instead of classic cell-by-cell backtracking, it aims to:

- Store row, column, and box constraints using integer bitmasks.
- Generate valid digit permutations inside each minigrid.
- Later combine these local permutations into a global solution.

## Features

- BitBoard class:
  - Tracks used digits per row, column, and minigrid via bitmasks.
  - O(1) checks for whether a digit can be placed in a cell.
  - Helpers to query available digits and empty cells per minigrid.
- Main module:
  - Encodes a sample 9×9 Sudoku as `list[list[int]]` (0 = empty).
  - Pretty-prints the board with 3×3 separators.
  - Counts and displays the number of empty cells.
  - Provides a starting CLI entrypoint for the solver.

## Project Structure

- `board.py`
  - Contains the `BitBoard` class and bitmask utilities.
- `main.py`
  - Defines the `PROBLEM` grid.
  - Contains `main()`.

More modules will be added for:

- Minigrid permutation generation.
- Combining minigrid permutations into full-board solutions.
- Benchmarking and experimentation with different strategies.
