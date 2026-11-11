This repository contains a Rust project for a command line application.
The application is still being developed. The goal is to generate extended
Markdown tables. The definition of this format is on
`./tmp/extended-markdown-table/README.md`.

Your next task is to implement a function `dump`. This function will take
a list of maps of string to string and output a string. This function will
be the opposite of `parse`: it should generate a Markdown table with the
data that is given to it.

This function should live in `src/parse.rs` alongside `parse`. You should
include automated tests to ensure that generation is correct.

Ask questions if you need clarifications.


