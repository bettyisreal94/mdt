This project is a Rust application for a command line tool to
handle extended Markdown tables as defined on
`./tmp/extended-markdown-table/README.md`.

The next task is the most crucial one so take your time and
read as much code as possible. The task consists of
implementing a function called `extend`. This function will
take two arguments: `inlet` and `extensions`. `inlet` is a list
of maps from string to string `Vec<HashMap<String, String>>`
from the output of `parse`. `extensions` is a list of pairs of
keys and extensions `Vec<(String, JsRule)>` from the output of
`extract_extensions`. This function will return a new list of
maps from string to string, the same shape as `inlet`.

For each entry in `inlet`, apply all extensions and associate
their result in the map with the key associated with that
extension. If any extension fails to execute, the whole
function should fail.

Everything in this task exists in `src/parser.rs`.
Implement new automated tests to ensure that this new function
works correctly. Ask questions if you need clarifications.

Don't change any of the existing functions, only implement the
new one.

