# Backtrace/Backtrait Error

Procedural macros to aid the propagation of `Backtrace`s between structs/enums (typically errors).

Dunno if it'd be useful yet.

## Gist

Provides function-like procedural macros to: generate a simple trait for facilitating propagating backtraces (the trait is defined where the macro is called); and, generate an enum that contains the backtrace and optional source of the error. See below.

```rust
define_backtrace_error!(ErrorWithBacktrace); // Expands into `pub trait ErrorWithBacktrace: std::error::Error {`...

define_backtrace_source!(BacktraceSource, ErrorWithBacktrace);
```

Provides an attribute procedural macro to derive a simple `std::error::Error` implementation for structs/enums. See below.

```rust
#[backtrace_derive(ErrorWithBacktrace)]
#[derive(Debug, BacktraceError)]
struct UnitError(#[display] String, #[backtrace] BacktraceSource);
```
