While describing configurations using a programming language is generally useful, there are situations where it's desirable that certain data is specified literally.

A notable example is cargo dependencies; by virtue of them being specified in a static TOML file, it is possible for tools like [cargo-edit](https://github.com/killercup/cargo-edit) to programmatically modify their versions.

In the Nix sphere, however, there exists a second desirable goal; the ability to describe how to build a software project in a single file (e.g. `flake.nix`) using composable programming constructs.

This crate defines a static file format with first-class support for embedding snippets of Nickel code. This would allow for accomplishing both these goals.

```ncl
# example.ncld

# Fields can only contain literal data by default

name = "example",

dependencies = {
  foo = "0.1.0",
},

# Snippets of Nickel code can be introduced using the $ prefix

outputs = $ fun dependencies => {
  world = dependencies.foo.bar,
  message = "Hello, %{world}!",
},
```