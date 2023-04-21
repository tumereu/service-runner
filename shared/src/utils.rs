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

#[macro_export]
macro_rules! write_escaped_str {
    ($fmt: tt, $string:expr) => {
        let mut escaped_str = $string.clone();
        escaped_str.replace("=", "\\=");
        escaped_str.replace("\"", "\\\"");

        if $string.contains(char::is_whitespace) || escaped_str.len() != $string.len() {
            $fmt.write_str("\"")?;
            $fmt.write_str(&escaped_str)?;
            $fmt.write_str("\"")?;
        } else {
            $fmt.write_str(&escaped_str)?;
        }
    }
}