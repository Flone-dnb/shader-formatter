# Shader Formatter

This is a standalone tool that accepts a path to a shader file to format. Returns `0` if formatted successfully, otherwise a non-zero value if an error occurred or something must be changed manually.

# Formatting rules

You can specify formatting rules by creating a file named `shader-formatter.toml` in the directory with your shaders or in any parent directory (similar to how you place a `.gitignore` file).

Here is an example `shader-formatter.toml` file:

```TOML
Indentation = "FourSpaces"
MaxEmptyLines = 1
LocalVariableCase = "Camel"
```

Below is the list of all possible formatting rules that you can describe in your `shader-formatter.toml`:

- **Indentation** - defines characters that will be used to indent lines of code.
    - Tab
    - TwoSpaces
    - FourSpaces
- **NewLineAroundOpenBraceRule** - defines whether to put a new line before an open brace or after it.
    - After
    - Before
- **MaxEmptyLines** - defines how much consecutive empty lines to keep (accepts any non-negative integer).
- **LocalVariableCase** - defines case style for local variables.
    - Camel
    - Pascal
    - Snake
    - UpperSnake

# Build

To build the tool you will need [Rust](https://www.rust-lang.org/tools/install).

Then in the root directory run:

```
cargo build --release
```

The compiled binary will be located at `/client/target/release/`.