extern crate proc_macro;
use proc_macro::{token_stream::IntoIter, Group, TokenStream, TokenTree};

#[proc_macro]
pub fn define_backtrace_error(trait_name: TokenStream) -> TokenStream {
    let trait_name = trait_name.into_iter().next();
    match trait_name {
        Some(TokenTree::Ident(trait_name)) => {
            format!("pub trait {trait_name}: std::error::Error {{\n\
                fn get_backtrace_source(&self) -> Option<&Box<dyn {trait_name}>>;\n\
                fn get_backtrace(&self) -> &Backtrace;\n\
                fn get_error(&self) -> &dyn std::error::Error;\n\
            }}").parse().expect("the only thing that could go wrong is a bad trait name")
        }
        _ => panic!("must supply name for the generated trait")
    }
}

#[proc_macro]
pub fn define_backtrace_source(arguments: TokenStream) -> TokenStream {
    let mut arguments = arguments.into_iter();
    let enum_name = arguments.next().expect("must supply name for the generated enum");
    arguments.next();
    let trait_name = arguments.next();
    match trait_name {
        Some(TokenTree::Ident(trait_name)) => {
            format!("\
                #[derive(Debug)]\n\
                enum {enum_name} {{\n\
                    Own(Backtrace, Option<Box<dyn std::error::Error>>),\n\
                    Other(Box<dyn {trait_name}>)\n\
                }}\n\
                impl {enum_name} {{\n\
                    fn new(underlying_error: Option<Box<dyn std::error::Error>>) -> Self {{\n\
                        Self::Own(Backtrace::capture(), underlying_error)\n\
                    }}\n\
                    fn from(underlying_error: Box<dyn {trait_name}>) -> Self {{\n\
                        Self::Other(underlying_error)\n\
                    }}\n\
                    fn get_backtrace_source(&self) -> Option<&Box<dyn {trait_name}>> {{\n\
                        match self {{\n\
                            {enum_name}::Own(_, _) => None,\n\
                            {enum_name}::Other(backtrace_error) => Some(&backtrace_error),\n\
                        }}\n\
                    }}\n\
                    fn get_backtrace(&self) -> &Backtrace {{\n\
                        match self {{\n\
                            {enum_name}::Own(backtrace, _) => &backtrace,\n\
                            {enum_name}::Other(backtrace_error) => {{\n\
                                backtrace_error.get_backtrace()\n\
                            }}\n\
                        }}\n\
                    }}\n\
                }}\
            ").parse().expect("the only thing that could go wrong is a bad trait name")
        }
        _ => panic!("must supply name of the backtrace error trait")
    }
}

#[proc_macro_derive(BacktraceError, attributes(display, backtrace, backtrace_source))]
pub fn permit_attributes(_: TokenStream) -> TokenStream {
    "".parse().expect("empty string failed to parse")
}

#[proc_macro_attribute]
pub fn backtrace_derive(attributes: TokenStream, mut struct_or_enum: TokenStream) -> TokenStream {
    let mut struct_or_enum_ = struct_or_enum.clone().into_iter();

    let trait_name = attributes.into_iter().next()
        .expect("need to supply name of the trait")
        .span().source_text()
        .expect("source text cannot be empty");
    let trait_name = trait_name.as_str();

    let trait_implementations = loop {
        match struct_or_enum_.next().and_then(|token| token.span().source_text()) {
            Some(token_text) if token_text.as_str() == "struct" => {
                break derive_for_struct(trait_name, struct_or_enum_);
            },
            Some(token_text) if token_text.as_str() == "enum" => {
                break derive_for_enum(trait_name, struct_or_enum_);
            },
            None => panic!("failed to find enum or struct keyword"),
            _ => {}
        };
    };
    
    struct_or_enum.extend(trait_implementations);
    struct_or_enum 
}

fn derive_for_struct(trait_name: &str, mut token_stream: IntoIter) -> TokenStream {
    let struct_name = token_stream.next().expect("struct name should follow struct keyword");
    let struct_body = token_stream.next();
    let (display_property, backtrace_property) = match struct_body {
        Some(TokenTree::Group(struct_body)) => {
            match struct_body.span_open().source_text().as_ref().map(String::as_str) {
                Some("{") => get_non_unit_struct_properties(struct_body),
                Some("(") => get_unit_struct_properties(struct_body),
                _ => panic!("encountered a bug when matching braces")
            }
        },
        _ => panic!("struct must have body"),
    };

    let display_implementation = display_property.map(|display_property| {
        format!("impl std::fmt::Display for {struct_name} {{\n\
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n\
                write!(f, \"{{}}\", self.{display_property})\n\
            }}\n\
        }}\n")
    }).unwrap_or(String::from(""));
    let backtrace_implementation = backtrace_property.map(|backtrace_property| {
        format!("\
            impl std::error::Error for {struct_name} {{\n\
                fn cause(&self) -> Option<&dyn std::error::Error> {{\n\
                    self.{backtrace_property}.get_backtrace_source().map(|error_source| error_source.get_error())\n\
                }}\n\
            }}\n\
            impl {trait_name} for {struct_name} {{\n\
                fn get_backtrace_source(&self) -> Option<&Box<dyn {trait_name}>> {{\n\
                    self.{backtrace_property}.get_backtrace_source()\n\
                }}\n\
                fn get_backtrace(&self) -> &Backtrace {{\n\
                    &self.{backtrace_property}.get_backtrace()\n\
                }}\n\
                fn get_error(&self) -> &dyn std::error::Error {{\n\
                    self\n\
                }}\n\
            }}\
        ")
    }).unwrap_or(String::from(""));
    format!("{display_implementation}{backtrace_implementation}").parse()
        .expect("failed to parse generated struct code")
}

fn get_non_unit_struct_properties(struct_body: Group) -> (Option<String>, Option<String>) {
    let mut display_property = None;
    let mut backtrace_property = None;

    let struct_body = struct_body.stream().into_iter().collect::<Vec<_>>();
    let mut struct_body = struct_body.windows(2);
    let mut does_display_exists = false;
    let mut does_backtrace_exists = false;
    while let Some([body_token_1, body_token_2]) = struct_body.next() {
        match (
            body_token_1.span().source_text().as_ref().map(String::as_str), 
            body_token_2.span().source_text().as_ref().map(String::as_str)
        ) {
            (Some("#"), Some("[display]")) => does_display_exists = true,
            (Some("#"), Some("[backtrace]")) => does_backtrace_exists = true,
            _ => {}
        }

        if let TokenTree::Ident(property_name) = &body_token_1 {
            if does_display_exists == true {
                display_property = Some(property_name.clone());
                does_display_exists = false;
            }
            if does_backtrace_exists == true {
                backtrace_property = Some(property_name.clone());
                does_backtrace_exists = false;
            }
        }
    };
    
    (
        display_property.as_ref().map(ToString::to_string), 
        backtrace_property.as_ref().map(ToString::to_string)
    )
}

fn get_unit_struct_properties(struct_body: Group) -> (Option<String>, Option<String>) {
    let mut display_property = None;
    let mut backtrace_property = None;

    let struct_body = struct_body.stream().into_iter().collect::<Vec<_>>();
    let mut struct_body = struct_body.windows(2);
    let mut does_display_exists = false;
    let mut does_backtrace_exists = false;
    let mut property_index = 0;
    let mut depth_level = 0;
    while let Some([body_token_1, body_token_2]) = struct_body.next() {
        match (
            body_token_1.span().source_text().as_ref().map(String::as_str), 
            body_token_2.span().source_text().as_ref().map(String::as_str) 
        ) {
            (Some("#"), Some("[display]")) => does_display_exists = true,
            (Some("#"), Some("[backtrace]")) => does_backtrace_exists = true,
            _ => {}
        }

        match (depth_level, body_token_1.span().source_text().as_ref().map(String::as_str)) {
            (0, Some(",")) => {
                if does_display_exists == true {
                    display_property = Some(property_index);
                    does_display_exists = false;
                }
                if does_backtrace_exists == true {
                    backtrace_property = Some(property_index);
                    does_backtrace_exists = false;
                }
                property_index += 1
            },
            (_, Some("<")) => depth_level += 1,
            (_, Some(">")) => depth_level -= 1,
            _ => {}
        }
    }
    if does_display_exists == true {
        display_property = Some(property_index);
    }
    if does_backtrace_exists == true {
        backtrace_property = Some(property_index);
    }

    (
        display_property.as_ref().map(ToString::to_string), 
        backtrace_property.as_ref().map(ToString::to_string)
    )
} 

fn derive_for_enum(trait_name: &str, mut token_stream: IntoIter) -> TokenStream {
    struct VariantInformation {
        name: String,
        display_property: Option<String>, 
        backtrace_property: Option<String> 
    }
    let mut variants_information = vec![];

    let enum_name = token_stream.next().expect("enum name should follow enum keyword");
    let enum_body = token_stream.next();
    match enum_body {
        Some(TokenTree::Group(enum_body)) => {
            let enum_body = enum_body.stream().into_iter().collect::<Vec<_>>();
            let mut enum_body = enum_body.windows(2);
            while let Some([body_token_1, body_token_2]) = enum_body.next() {
                match (body_token_1, body_token_2) {
                    (TokenTree::Ident(variant_name), TokenTree::Group(variant_body)) => {
                        let (display_property, backtrace_property) = match variant_body.span_open().source_text().as_ref().map(String::as_str) {
                            Some("(") => get_unit_struct_properties(variant_body.clone()),
                            Some("{") => get_non_unit_struct_properties(variant_body.clone()),
                            _ => panic!("variants must have a body"),
                        };

                        variants_information.push(VariantInformation {
                            name: variant_name.span().source_text().expect("identifiers should have non-none source text"),
                            display_property,
                            backtrace_property,
                        });
                    },
                    _ => {}
                }
            }
        },
        _ => panic!("could not find enum body")
    }

    if !(variants_information.iter().all(|info| info.display_property.is_none()) ||
    variants_information.iter().all(|info| info.display_property.is_some())) {
        panic!("display attribute must be applied to none of the variant or all");
    }

    if !(variants_information.iter().all(|info| info.backtrace_property.is_none()) ||
    variants_information.iter().all(|info| info.backtrace_property.is_some())) {
        panic!("backtrace attribute must be applied to none of the variant or all");
    }

    let generate_arm = |variant_name, property_name: String, lhs_prefix, rhs_prefix, rhs_suffix| {
        if let Ok(property_index) = property_name.parse::<usize>() {
            let pattern_padding = "_, ".repeat(property_index);
            format!("{enum_name}::{variant_name}({pattern_padding}{lhs_prefix}property_name, ..) => {rhs_prefix}property_name{rhs_suffix}")
        } else {
            format!("{enum_name}::{variant_name} {{ {lhs_prefix}{property_name}, .. }} => {rhs_prefix}{property_name}{rhs_suffix}")
        }
    };

    let display_implementation = variants_information.iter().map(|info| {
            let property_name = info.display_property.clone()?;
            Some(generate_arm(info.name.clone(), property_name, "", "", ""))
        })
        .collect::<Option<Vec<_>>>()
        .map(|display_arms| {
            let display_arms = display_arms.join(",\n");
            format!("impl std::fmt::Display for {enum_name} {{\n\
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n\
                    write!(f, \"{{}}\", match self {{\n\
                        {display_arms}\n\
                    }})\n\
                }}\n\
            }}\n")
        })
        .unwrap_or(String::from(""));

    let backtrace_implementation = variants_information.iter()
        .try_fold((String::new(), String::new(), String::new()), |mut accumulator, info| {
            let property_name = info.backtrace_property.clone()?;
            
            accumulator.0 += &generate_arm(
                info.name.clone(), property_name.clone(), 
                "", "", ".get_backtrace_source().map(|error_source| error_source.get_error())"
                
            );
            accumulator.0 += ",\n";
            accumulator.1 += &generate_arm(
                info.name.clone(), property_name.clone(), 
                "", "", ".get_backtrace_source()"
            );
            accumulator.1 += ",\n";
            accumulator.2 += &generate_arm(
                info.name.clone(), property_name, 
                "", "&", ".get_backtrace()"
            );
            accumulator.2 += ",\n";
            Some(accumulator)
        })
        .map(|(mut cause_arms, mut source_arms, mut backtrace_arms)| {
            cause_arms.pop();
            source_arms.pop();
            backtrace_arms.pop();
            format!("\
                impl std::error::Error for {enum_name} {{\n\
                    fn cause(&self) -> Option<&dyn std::error::Error> {{\n\
                        match self {{\n\
                            {cause_arms}\n\
                        }}\n\
                    }}\n\
                }}\n\
                impl {trait_name} for {enum_name} {{\n\
                    fn get_backtrace_source(&self) -> Option<&Box<dyn {trait_name}>> {{\n\
                        match self {{\n\
                            {source_arms}\n\
                        }}\n\
                    }}\n\
                    fn get_backtrace(&self) -> &Backtrace {{\n\
                        match self {{\n\
                            {backtrace_arms}\n\
                        }}\n\
                    }}\n\
                    fn get_error(&self) -> &dyn std::error::Error {{\n\
                        self\n\
                    }}\n\
                }}\
            ")
        })
        .unwrap_or(String::from(""));

    format!("{display_implementation}{backtrace_implementation}").parse()
        .expect("failed to parse generated enum code")
}
