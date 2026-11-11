This repository contains a Rust project for a command line application. The application is
still being developed. The goal is to generate extended Markdown tables. The definition of this
format is on `./tmp/extended-markdown-table/README.md`.

Your next task is to implement a function called `str_to_fn`. This function will take a string
as input and parse the code in it into a closure. The code will always be in Javascript. To
parse these functions, use the package available on <https://github.com/boa-dev/boa>.

During implementation, include automated tests for:

- Failure to parse the function
- Correctly parsing the function and using it

Ask questions if you need clarifications.

