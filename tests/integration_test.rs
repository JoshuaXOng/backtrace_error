use std::backtrace::Backtrace;
use backtrait_error::{backtrace_derive, define_backtrace_error, define_backtrace_source, BacktraceError};

#[test]
fn test_crate() {
    define_backtrace_error!(ErrorWithBacktrace);
    define_backtrace_source!(BacktraceSource, ErrorWithBacktrace);

    #[derive(Debug, BacktraceError)]
    #[backtrace_derive(ErrorWithBacktrace)]
    struct AnError(#[display] String, #[backtrace] BacktraceSource);
    impl AnError {
        fn new(error_message: impl Into<String>, underlying_error: Option<Box<dyn std::error::Error>>) -> Self {
            Self(error_message.into(), BacktraceSource::new(underlying_error))
        }
    }
    let _e = AnError::new("what went wrong?", Some(std::io::Error::other("oh no!").into()));
    let e_1 = AnError::new("a new error", None);
    let e_2 = AnError(String::from("storing the new error"), BacktraceSource::from(Box::new(e_1)));
    println!("{e_2}");
    println!("{}", e_2.get_backtrace());
    println!("{e_2:#?}");

    #[derive(Debug, BacktraceError)]
    #[backtrace_derive(ErrorWithBacktrace)]
    struct StructError_ {
        #[display] 
        message: String,
        #[backtrace] 
        backtrace_source: BacktraceSource 
    }
    let e = StructError_ { message: String::from("sigh"), backtrace_source: BacktraceSource::new(None) };
    println!("{e}");
    println!("{e:#?}");

    #[backtrace_derive(ErrorWithBacktrace)]
    #[derive(Debug, BacktraceError)]
    struct StructError {
        #[display] 
        message: String,
        #[backtrace] 
        backtrace_source: BacktraceSource
    }
    let e = StructError { message: String::from("not bad"), backtrace_source: BacktraceSource::new(None) };
    println!("{e}");
    println!("{e:#?}");

    #[backtrace_derive(ErrorWithBacktrace)]
    #[derive(Debug, BacktraceError)]
    struct UnitError(#[display] String, #[backtrace] BacktraceSource, #[allow(dead_code)] Result<(), ()>);
    let e = UnitError(String::from("at all"), BacktraceSource::new(None), Ok(()));
    println!("{e}");
    println!("{e:#?}");

    #[backtrace_derive(ErrorWithBacktrace)]
    #[derive(Debug, BacktraceError)]
    enum EnumError {
        ABitScuffed(#[display] String, #[backtrace] BacktraceSource),
        OnTheMacroScale(#[display] String, #[backtrace] BacktraceSource, #[allow(dead_code)] Result<(), ()>),
        #[allow(dead_code)]
        Eh(#[allow(dead_code)] Result<(), ()>, #[display] String, #[backtrace] BacktraceSource),

        #[allow(dead_code)]
        StructVariant {
            #[display] 
            message: String,
            #[backtrace] 
            backtrace: BacktraceSource,
        }
    }
    let e = EnumError::ABitScuffed(String::from("bah"), BacktraceSource::new(None));
    println!("{e}");
    println!("{e:#?}");
    let e = EnumError::OnTheMacroScale(String::from("brain"), BacktraceSource::new(None), Ok(()));
    println!("{e}");
    println!("{e:#?}");
    e.get_backtrace();

    #[backtrace_derive(ErrorWithBacktrace)]
    #[derive(Debug, BacktraceError)]
    struct NoAttributeError(String, #[backtrace] BacktraceSource, #[allow(dead_code)] Result<(), ()>);
    impl std::fmt::Display for NoAttributeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0) 
        }
    }
    let e = NoAttributeError(String::from("hurts"), BacktraceSource::new(None), Ok(()));
    println!("{e}");
    println!("{e:#?}");
}
