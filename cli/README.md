# `seesaw`
```text
Generate a trait from `extern "C"` blocks, typically emitted by bindgen

Usage: seesaw [OPTIONS] <NAME> [BINDINGS]

Arguments:
  <NAME>
          The name of the trait

  [BINDINGS]
          Path the the input file. If absent or `-`, read from stdin

Options:
  -a, --allow <ALLOW>
          Regexes of function names to include. By default, all functions are allowed

  -b, --block <BLOCK>
          Regexes of function names to exclude
```