use std::backtrace::Backtrace;
use backtrace_error::{define_backtrace_error, BacktraceError};

#[test]
fn test_crate() {
    define_backtrace_error!(BacktraceError);

    #[derive(Debug, BacktraceError)]
    struct StructError {
        #[display] 
        message: String,
        #[backtrace] 
        backtrace: Backtrace
    }
    let e = StructError { message: String::from("not bad"), backtrace: Backtrace::capture() };
    println!("{e}");
    println!("{e:#?}");

    #[derive(Debug, BacktraceError)]
    struct UnitError(#[display] String, #[backtrace] Backtrace, #[allow(dead_code)] Result<(), ()>);
    let e = UnitError(String::from("at all"), Backtrace::capture(), Ok(()));
    println!("{e}");
    println!("{e:#?}");

    #[derive(Debug, BacktraceError)]
    enum EnumError {
        ABitScuffed(#[display] String, #[backtrace] Backtrace),
        OnTheMacroScale(#[display] String, #[backtrace] Backtrace, #[allow(dead_code)] Result<(), ()>),
        #[allow(dead_code)]
        Eh(#[allow(dead_code)] Result<(), ()>, #[display] String, #[backtrace] Backtrace),

        #[allow(dead_code)]
        StructVariant {
            #[display] 
            message: String,
            #[backtrace] 
            backtrace: Backtrace,
        }
    }
    let e = EnumError::ABitScuffed(String::from("bah"), Backtrace::capture());
    println!("{e}");
    println!("{e:#?}");
    let e = EnumError::OnTheMacroScale(String::from("brain"), Backtrace::capture(), Ok(()));
    println!("{e}");
    println!("{e:#?}");

    #[derive(Debug, BacktraceError)]
    struct NoAttributeError(String, #[backtrace] Backtrace, #[allow(dead_code)] Result<(), ()>);
    impl std::fmt::Display for NoAttributeError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0) 
        }
    }
    let e = NoAttributeError(String::from("hurts"), Backtrace::capture(), Ok(()));
    println!("{e}");
    println!("{e:#?}");
}
