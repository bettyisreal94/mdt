This repository contains a Rust project for a command line application that handles extended
Markdown tables as defined `./tmp/extended-markdown-tables/README.md`.

The current task is to fix a bug that exists throughout the flow on the `process` function in
`./src/parser.rs`. Currently, the columns in the output table are sorted. However, I would like
them to keep the same original order. This means then that a table defined like this:

```
| name   | role      |
|--------|-----------|
| kurisu | bassist   |
| betty  | guitarist |
: job = (t, r, i) => `r["name"] the r["role"]`
```

Should be output like this:

```
| name   | role      | job                 |
|--------|-----------|---------------------|
| kurisu | bassist   | kurisu the bassist  |
| betty  | guitarist | betty the guitarist |
```

Review the code and propose a plan to address this bug. Write the plan to `TODO.md` with
detailed implementation steps. Only the ordering of the columns should be handled in this task.

