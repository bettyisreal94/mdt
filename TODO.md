This repository contains a command line tool written in Rust. The next task
is to implement a function will receive a string and either a list of maps
with data parsed from that string or an error. The expectation is that
this string will contain an extended markdown table, defined on
`tmp/extended-markdown-table/README.md`. Implement the function as
described. Include unit tests for an expected case based on the definition
provided and another case where parsing fails. For now, ignore the extension
lines as they will be parsed in a future effort. The unit tests should be
executed from make as `make test`.

