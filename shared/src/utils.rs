/// Prints out a line to stdout, but only when the binary has been compiled in debug mode
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

#[macro_export]
macro_rules! format_err {
    ($msg:expr, $err:expr) => {
        {
            use ::std::error::Error;

            let mut error_opt: ::std::option::Option<&dyn ::std::error::Error> = ::std::option::Option::Some(&$err);
            let mut message: ::std::string::String = $msg.into();
            while let ::std::option::Option::Some(error) = error_opt {
                message.push_str(::std::format!(": {error}").as_str());
                error_opt = error.source();
            }

            message
        }
    };
}