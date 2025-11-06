you are currently in the /code directory.
the task is to migrate the ./neuramancer project from 0.6.3 to 0.7.2.
the current project dependency is available in ./neuramancer/flake.nix and ./neuramancer/Cargo.toml.
use the local ./lib/dioxus source code to inform edits.
`cargo check` is available with `nix develop`:
```sh
cd neuramancer && nix develop --extra-experimental-features "nix-command flakes" --impure --show-trace --command cargo check 2>&1 | grep -A 5 "error\["
```

fix the error first, and then focus on patterns/styles indicated by the ./lib/dioxus source code.
