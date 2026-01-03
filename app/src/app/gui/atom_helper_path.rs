use std::borrow::Cow;
use std::path::Path;

#[cfg(windows)]
fn normalize_windows_path_prefix_for_display(s: &str) -> Cow<'_, str> {
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return Cow::Owned(format!(r"\\{rest}"));
    }
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        return Cow::Borrowed(rest);
    }
    Cow::Borrowed(s)
}

#[cfg(not(windows))]
fn normalize_windows_path_prefix_for_display(s: &str) -> Cow<'_, str> {
    Cow::Borrowed(s)
}

pub fn normalize_windows_path_prefixes_in_text(s: &str) -> Cow<'_, str> {
    #[cfg(windows)]
    {
        if !s.contains(r"\\?\") {
            return Cow::Borrowed(s);
        }

        let mut out = s.to_string();
        out = out.replace(r"\\?\UNC\", r"\\");
        out = out.replace(r"\\?\", "");
        Cow::Owned(out)
    }
    #[cfg(not(windows))]
    {
        Cow::Borrowed(s)
    }
}

pub fn display_path(path: &Path) -> String {
    let raw = path.display().to_string();
    normalize_windows_path_prefix_for_display(&raw).into_owned()
}

