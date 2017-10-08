/// Parser method generator.
///
/// Invoke inside an impl block. Declare a list of signatures of methods callable on `&mut self`.
/// The macro constructs a `parse(&mut self, input: &str) -> Result<(), String>` method that parses
/// input strings like "method_name 4 foo" into calls of `self.method_name(FromStr::from_str("4")?,
/// FromStr::from_str("foo")?)`.
///
///     # #[macro_use] extern crate calx_alg;
///     # fn main() {
///
///     struct Adder {
///         pub num: u32,
///     }
///
///     impl Adder {
///         fn add(&mut self, amount: u32) {
///             self.num += amount;
///         }
///
///         command_parser!{
///             fn add(&mut self, amount: u32);
///         }
///     }
///
///     let mut a = Adder { num: 1 };
///     a.add(2);
///     assert_eq!(a.num, 3);
///
///     assert!(a.parse("foobie bletch").is_err());
///     assert!(a.parse("add").is_err());       // Too few arguments
///     assert!(a.parse("add derp").is_err());  // Arguments won't parse to unsigned integer.
///     assert!(a.parse("add -1").is_err());
///     assert!(a.parse("add 1.0").is_err());
///
///     // NB: Due to limits of the macro implementation, trailing arguments
///     // are ignored instead of causing an error.
///     // assert!(a.parse("add 1 2").is_err());   // Too many arguments
///
///     assert_eq!(a.num, 3);
///
///     assert!(a.parse("add 3").is_ok());
///     assert_eq!(a.num, 6);
///
///     # }
#[macro_export]
macro_rules! command_parser {
    {
        // TODO: Should support immutable self methods too.
        $(fn $method:ident(&mut self$(, $argname:ident: $argtype:ty)*);)*
    } => {
        #[allow(unused)]
        fn parse(&mut self, input: &str) -> ::std::result::Result<(), Box<::std::error::Error>> {
            #[derive(Debug)]
            struct ParseError(String);

            impl ::std::fmt::Display for ParseError {
                fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                    self.0.fmt(fmt)
                }
            }

            impl ::std::error::Error for ParseError {
                fn description(&self) -> &str { &self.0 }
            }

            fn parse_err(s: String) -> ::std::result::Result<(), Box<::std::error::Error>> {
                Err(Box::new(ParseError(s)))
            }

            use ::std::str::FromStr;
            let input = input.trim_right();
            let mut elts = input.split(" ");
            let cmd = elts.next();
            if let Some(cmd) = cmd {
                match cmd {
                    $(
                    stringify!($method) => {
                        self.$method(
                            $({
                                if let Some(arg) = elts.next() {
                                    let param: $argtype = FromStr::from_str(arg)?;
                                    param
                                } else {
                                    return parse_err("Too few arguments".to_string());
                                }
                            }),*
                        );
                        Ok(())
                    }
                    )*
                    x => parse_err(format!("Unknown command {}", x)),
                }
            } else {
                return parse_err("No input".to_string());
            }
        }
    }
}
