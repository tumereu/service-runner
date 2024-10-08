macro_rules! write_escaped_str {
    ($fmt: tt, $string:expr) => {
        let escaped_str = $string.clone();
        let escaped_str = escaped_str.replace("=", "\\=");
        let escaped_str = escaped_str.replace("\"", "\\\"");

        if $string.contains(char::is_whitespace) || escaped_str.len() != $string.len() {
            $fmt.write_str("\"")?;
            $fmt.write_str(&escaped_str)?;
            $fmt.write_str("\"")?;
        } else {
            $fmt.write_str(&escaped_str)?;
        }
    };
}

macro_rules! format_err {
    ($msg:expr, $err:expr) => {{
        let mut error_opt: ::std::option::Option<&dyn ::std::error::Error> =
            ::std::option::Option::Some(&$err);
        let mut message: ::std::string::String = $msg.into();
        while let ::std::option::Option::Some(error) = error_opt {
            message.push_str(::std::format!(": {error}").as_str());
            error_opt = error.source();
        }

        message
    }};
}

pub (crate) use write_escaped_str;
pub (crate) use format_err;
