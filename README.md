# Backtrace/Backtrait Error

Procedural macros to aid the propagation of `Backtrace`s between structs/enums (typically errors).

Dunno if it'd be useful yet.

## Gist

Provides a function-like procedural macro to generate a simple trait for facilitating propagating backtraces. The trait is defined where the macro is called. See below.

```rust
define_backtrace_error!(ErrorWithBacktrace); // Expands into `pub trait ErrorWithBacktrace: std::error::Error {`...
```

Provides an attribute procedural macro to derive a simple `std::error::Error` implementation for structs/enums. See below.

```rust
#[backtrace_derive(ErrorWithBacktrace)]
#[derive(Debug, BacktraceError)]
struct UnitError(#[display] String, #[backtrace] Backtrace);
```
