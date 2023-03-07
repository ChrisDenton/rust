use crate::env;
use crate::ffi::{CStr, OsStr, OsString};
use crate::io;
use crate::os::unix::ffi::OsStringExt;
use crate::path::{Path, PathBuf, Prefix};

pub type NativePath = CStr;
pub use crate::sys::common::small_c_string::run_path_with_cstr;

#[unstable(feature = "path_like_internals", issue = "none")]
pub trait PathLike: crate::sealed::Sealed {
    fn with_path<T, F: FnOnce(&Path) -> io::Result<T>>(self, f: F) -> io::Result<T>;
    fn with_native_path<T, F: FnOnce(&NativePath) -> io::Result<T>>(self, f: F) -> io::Result<T>;
}

#[unstable(feature = "path_like_internals", issue = "none")]
impl<P: AsRef<Path>> PathLike for P {
    fn with_path<T, F: FnOnce(&Path) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        f(self.as_ref())
    }
    fn with_native_path<T, F: FnOnce(&NativePath) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        run_path_with_cstr(self.as_ref(), f)
    }
}

#[unstable(feature = "path_like_internals", issue = "none")]
impl PathLike for &NativePath {
    fn with_path<T, F: FnOnce(&Path) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        let path = PathBuf::from(OsString::from_vec(self.to_bytes().to_vec()));
        f(&path)
    }
    fn with_native_path<T, F: FnOnce(&NativePath) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        f(self)
    }
}

#[unstable(feature = "path_like_internals", issue = "none")]
impl PathLike for &crate::path::NativePath {
    fn with_path<T, F: FnOnce(&Path) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        self.0.with_path(f)
    }
    fn with_native_path<T, F: FnOnce(&NativePath) -> io::Result<T>>(self, f: F) -> io::Result<T> {
        self.0.with_native_path(f)
    }
}
#[unstable(feature = "sealed", issue = "none")]
impl<P: AsRef<Path>> crate::sealed::Sealed for P {}
#[unstable(feature = "sealed", issue = "none")]
impl crate::sealed::Sealed for &NativePath {}

#[inline]
pub fn is_sep_byte(b: u8) -> bool {
    b == b'/'
}

#[inline]
pub fn is_verbatim_sep(b: u8) -> bool {
    b == b'/'
}

#[inline]
pub fn parse_prefix(_: &OsStr) -> Option<Prefix<'_>> {
    None
}

pub const MAIN_SEP_STR: &str = "/";
pub const MAIN_SEP: char = '/';

/// Make a POSIX path absolute without changing its semantics.
pub(crate) fn absolute(path: &Path) -> io::Result<PathBuf> {
    // This is mostly a wrapper around collecting `Path::components`, with
    // exceptions made where this conflicts with the POSIX specification.
    // See 4.13 Pathname Resolution, IEEE Std 1003.1-2017
    // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap04.html#tag_04_13

    // Get the components, skipping the redundant leading "." component if it exists.
    let mut components = path.strip_prefix(".").unwrap_or(path).components();
    let path_os = path.as_os_str().bytes();

    let mut normalized = if path.is_absolute() {
        // "If a pathname begins with two successive <slash> characters, the
        // first component following the leading <slash> characters may be
        // interpreted in an implementation-defined manner, although more than
        // two leading <slash> characters shall be treated as a single <slash>
        // character."
        if path_os.starts_with(b"//") && !path_os.starts_with(b"///") {
            components.next();
            PathBuf::from("//")
        } else {
            PathBuf::new()
        }
    } else {
        env::current_dir()?
    };
    normalized.extend(components);

    // "Interfaces using pathname resolution may specify additional constraints
    // when a pathname that does not name an existing directory contains at
    // least one non- <slash> character and contains one or more trailing
    // <slash> characters".
    // A trailing <slash> is also meaningful if "a symbolic link is
    // encountered during pathname resolution".
    if path_os.ends_with(b"/") {
        normalized.push("");
    }

    Ok(normalized)
}
