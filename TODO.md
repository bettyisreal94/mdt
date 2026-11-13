This project is a Rust project for a command line application for dealing
with extended Markdown tables. The application is still under development.

The next task is to update the `extract_extensions` to use `str_to_fn`.
`extract_entensions` extract extensions from tables. Currently these
extractions return a list of string pairs `Vec(String, String)`, where the
first string is the extension key and the second string is the extension
code. The goal is to return `Vec(String, JsRule)` by using `str_to_fn` to
convert the string into a closure.

If converting any code into closure fails, the whole `extract_extensions`
function should fail. Update the relevant automated tests accordingly.

`extract_extensions` lives in `src/parser.rs`. `str_to_fn` lives in
`src/js.rs`.

Ask questions if you need clarifications.

