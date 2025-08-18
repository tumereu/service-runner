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

use std::path::Path;
pub (crate) use write_escaped_str;
pub (crate) use format_err;

/// Resolves the given (user defined) path as a proper path object. If the path is relative, then it is treated as a
/// relative path directly under the given workdir. If the path-input is absolute, then workdir is ignored and the path
/// is resolved as is.
pub fn resolve_path(path: &str, workdir: &str) -> ::std::path::PathBuf {
    let path_input = Path::new(&path);
    if path_input.is_absolute() {
        path_input.to_path_buf()
    } else {
        Path::new(&workdir).join(path_input)
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_resolve_path_cross_platform() {
        let result = resolve_path("file.txt", "workdir");
        assert_eq!(result, Path::new("workdir").join("file.txt"));

        let result = resolve_path("docs/report.md", "tmp/work");
        assert_eq!(result, Path::new("tmp/work/docs/report.md"));

        let result = resolve_path("", "var/tmp");
        assert_eq!(result, Path::new("var/tmp"));

        let result = resolve_path("./../logs/app.log", "srv/app");
        assert_eq!(result, Path::new("srv/app/./../logs/app.log"));
    }

    #[cfg(unix)]
    #[test]
    fn test_resolve_paths_unix() {
        let result = resolve_path("/etc/config.yaml", "/ignored/workdir");
        assert_eq!(result, Path::new("/etc/config.yaml"));

        let result = resolve_path("script.sh", "/opt/");
        assert_eq!(result, Path::new("/opt/script.sh"));

        let result = resolve_path("/", "/workdir");
        assert_eq!(result, PathBuf::from("/"));
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_paths_windows() {
        let result = resolve_path(r"C:\Windows\System32\drivers\etc\hosts", r"C:\ignored");
        assert_eq!(result, Path::new(r"C:\Windows\System32\drivers\etc\hosts"));

        let result = resolve_path(r"docs\report.md", r"C:\Work");
        assert_eq!(result, Path::new(r"C:\Work\docs\report.md"));

        let result = resolve_path(r"C:\", r"D:\ignored");
        assert_eq!(result, PathBuf::from(r"C:\"));

        let result = resolve_path("script.bat", r"C:\opt\");
        assert_eq!(result, Path::new(r"C:\opt\script.bat"));
    }
}
