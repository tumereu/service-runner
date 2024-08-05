use crate::model::message::models::{OutputKey, OutputStore};
use crate::model::system_state::SystemState;

/// Prints out a line to stdout, but only when the binary has been compiled in debug mode
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

#[macro_export]
macro_rules! format_err {
    ($msg:expr, $err:expr) => {{
        use ::std::error::Error;

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

#[macro_export]
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

pub fn get_active_outputs<'a>(store: &'a OutputStore, state: &'a Option<SystemState>) -> Vec<&'a OutputKey> {
    store.outputs.iter()
        .map(|(key, _)| key)
        .filter(|key| {
            state.as_ref().map(|state| {
                state.service_statuses
                    .get(key.service_ref.as_str())
                    .map(|status| status.show_output)
            }).flatten().unwrap_or(false)
        }).collect()
}