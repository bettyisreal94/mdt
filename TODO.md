This repository contains a Rust project for a command line application that
handles extended Markdown tables as defined on
`./tmp/extended-markdown-table/README.md`.

This current task is to fix a bug that happens on the `extend` function in
`./src/parser.rs`. If an extension function contains a recursive function,
the results of previous function calls are not stored so the data is not
generated correctly.

As an example, consider the following table:

```
| what    | how much |
|---------|----------|
| salary  |     4000 |
| food    |     -200 |
| rent    |     -800 |
| candles |    -3600 |
| utility |     -150 |
: total = (t, r, i) => parseInt(r["how much"]) + ((i === 0)? 0 : parseInt(t[i - 1]["total"]))
```

The `total` column should be calculated using previously obtained results
but the application has `NaN` on all rows except the first.

Review the function definition and address this bug. Include an automated
test to take this case into consideration too.

