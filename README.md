# Shader Formatter

This is a standalone tool that accepts a path to a shader file to format. Returns `0` if formatted successfully, otherwise a non-zero value if an error occurred or something must be changed manually.

Note
> Only HLSL and GLSL are supported.

This tool combines features from such tools as `clang-format`, `clang-tidy` and `doxygen`.

# VSCode extension

There is a VSCode extension for this tool, see: https://github.com/Flone-dnb/vscode-shader-formatter

# Neovim plugin

There is a Neovim plugin for this tool, see: https://github.com/Flone-dnb/shader-formatter.nvim

# Formatting rules

You can specify formatting rules by creating a file named `shader-formatter.toml` in the directory with your shaders or in any parent directory (similar to how you place a `.gitignore` file).

Here is an example `shader-formatter.toml` file:

```TOML
Indentation = "FourSpaces"
MaxEmptyLines = 1
SpacesInBrackets = true
VariableCase = "Camel"
```

Below is the list of all possible formatting rules that you can describe in your `shader-formatter.toml`:

- **Indentation** (string) - defines characters that will be used to indent lines of code.
    - Tab
    - TwoSpaces
    - FourSpaces
- **NewLineOnOpenBrace** (string) - defines whether to put a new line before an open brace or after it.
    - After
    - Before
- **MaxEmptyLines** (unsigned integer) - defines how much consecutive empty lines to keep.
- **SpacesInBrackets** (boolean) - defines whether or not to add spaces between `(` and `)`, `[` and `]` for example: if enabled converts `foo(param1, param2)` to `foo( param1, param2 )`.
- **IndentPreprocessor** (bool) - defines whether preprocessor directives will be indented or not.
- **PreprocessorIfCreatesNesting** (bool) - defines whether or not preprocessor directives such as `#if`, `#ifdef`, `#elif` and `#else` create nesting just like regular `if`/`else` keywords. Only works when `IndentPreprocessor` is enabled.
- **RequireDocsOnFunctions** (bool) - defines whether documentation comments on functions are required or not. Here are a few examples of documentation comments:

```
/**
* Function docs.
*
* @param value Input value docs.
*
* @return Return value docs.
*/
int foo(int value) {}

// Function docs.
// 
// @param value Input value docs.
// 
// @return Return value docs.
int foo(int value) {}
```

- **RequireDocsOnStructs** (bool) - defines whether documentation comments on structs are required or not.
- **RequireDocsOnFields** (bool) - defines whether documentation comments on struct fields are required or not.

Below are the rules that are not checked unless they are specified in your configuration file:

- **VariableCase** (string) - defines case style for variables.
    - Camel
    - Pascal
    - Snake
    - UpperSnake
- **FunctionCase** (string) - defines case style for functions (options are the same as in "variable case" rule).
- **StructCase** (string) - defines case style for structs (options are the same as in "variable case" rule).
- **BoolPrefix** (string) - defines required prefix for `bool` variables, for example if this rule is set to `b` then a correct variable may look like this: `bValue`.
- **IntPrefix** (string) - defines required prefix for integer variables, for example if this rule is set to `i` then a correct variable may look like this: `iValue`.
- **FloatPrefix** (string) - defines required prefix for floating-point variables, for example if this rule is set to `f` then a correct variable may look like this: `fValue`.
- **GlobalVariablePrefix** (string) - defines required prefix for global variables, this rule is applied before other prefix and case rules so you can have a "mixed" global variables names like "g_iMyVariable" where global prefix is "g_", int prefix is "i" and case is "Camel".

- **ForceLineEnding** (string) - custom line ending, by default the formatter uses line ending according to the OS ("\r\n" for Windows and "\n" for everything else) but you can overwrite this setting by specifying the value, for example: "\n".

# Temporary disabling formatting or checks

Similar to `clang-tidy` you can use `NOLINT` comments to disable checks for certain parts of your code:

```
int WrOnG_cAsE = 1; // NOLINT

// NOLINTBEGIN
int WrOnG_cAsE = 1;
// NOLINTEND
```

In addition, similar to `clang-format`s `clang-format off` comments you can mark some parts of your code to be skipped by the formatted:

```
// NOFORMATBEGIN
void foo(            int a);
void bar( SomeCustomType b);
// NOFORMATEND
```

# Command line options

There are some command line options that you might find useful such as running the formatter to only check if formatting is needed or not (without formatting the actual file). Run the tool without any arguments to see available command line options.

# Build

To build the tool you will need [Rust](https://www.rust-lang.org/tools/install).

Then in the root directory run:

```
cargo build --release
```

The compiled binary will be located at `/target/release/`.
