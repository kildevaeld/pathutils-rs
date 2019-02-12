use std::path::{Path, PathBuf};
use std::str::Chars;
mod error;

pub use error::*;

#[cfg(feature = "glob")]
pub mod glob;

pub fn join<T: AsRef<Path>, S: AsRef<str>>(base: T, cmp: S) -> Result<PathBuf> {
    let mut path = cmp.as_ref();
    let mut base = base.as_ref();
    if path.starts_with("./") {
        path = path.trim_left_matches("./");
    } else if path == "." {
        return Ok(base.to_path_buf());
    } else if path.starts_with("/") {
        path = path.trim_left_matches("/");
    }

    while path.starts_with("..") {
        base = match base.parent() {
            Some(parent) => parent,
            None => return Err(Error::new(ErrorKind::Unknown)),
        };
        path = path.trim_left_matches("..");
        if !path.is_empty() && path.chars().nth(0).unwrap() == '/' {
            path = path.trim_left_matches("/");
        }
    }

    Ok(base.join(path))
}

pub fn join_slice<T: AsRef<Path>, S: AsRef<str>>(base: T, cmps: &[S]) -> Result<PathBuf> {
    let mut path = base.as_ref().to_path_buf();
    for p in cmps {
        path = join(&path, p)?;
    }
    Ok(path)
}

pub fn extname<T: AsRef<str>>(filename: T) -> Option<String> {
    if filename.as_ref() == "." || filename.as_ref() == ".." {
        return None;
    }
    let iter: String = filename
        .as_ref()
        .chars()
        .rev()
        .take_while(|m| *m != '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if iter.as_str() == filename.as_ref() {
        None
    } else {
        Some(iter)
    }
}

pub fn parent<T: AsRef<str>>(filename: T) -> Option<String> {
    if filename.as_ref().is_empty()
        || filename.as_ref() == "/"
        || filename.as_ref() == "."
        || filename.as_ref() == ".."
    {
        return None;
    }

    match filename.as_ref().rfind('/') {
        Some(idx) => Some(filename.as_ref().chars().take(idx + 1).collect()),
        None => Some("".to_owned()),
    }
}

pub fn filename<T: AsRef<str>>(path: T) -> Option<String> {
    if path.as_ref().is_empty()
        || path.as_ref() == "/"
        || path.as_ref() == "."
        || path.as_ref() == ".."
    {
        return None;
    }

    match path.as_ref().rfind('/') {
        Some(idx) => Some(path.as_ref().chars().skip(idx + 1).collect()),
        None => Some(path.as_ref().to_string()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn join_test() {
        let path = Path::new("/test");
        assert_eq!(
            join(path, "test/me.txt").unwrap(),
            PathBuf::from("/test/test/me.txt")
        );

        assert_eq!(join(path, "../me.txt").unwrap(), PathBuf::from("/me.txt"));
        assert_eq!(join(path, ".").unwrap(), PathBuf::from("/test"));
        assert_eq!(join(path, "..").unwrap(), PathBuf::from("/"));
        assert_eq!(join(path, "../../me.txt").is_err(), true);
    }

    #[test]
    fn extname_test() {
        assert_eq!(extname("test.exe"), Some(String::from("exe")));
        assert_eq!(extname("yggdrasil/test.exe"), Some(String::from("exe")));
        assert_eq!(extname("yggdrasil/test"), None);
        assert_eq!(extname("."), None);
        assert_eq!(extname(".."), None);
    }

    #[test]
    fn parent_test() {
        assert_eq!(parent("test.exe"), Some(String::from("")));
        assert_eq!(
            parent("yggdrasil/test.exe"),
            Some(String::from("yggdrasil/"))
        );
        assert_eq!(
            parent("yggdrasil/test/visse"),
            Some(String::from("yggdrasil/test/"))
        );
        assert_eq!(parent("."), None);
        assert_eq!(parent(".."), None);
    }

    #[test]
    fn filename_test() {
        assert_eq!(filename("test.exe"), Some(String::from("test.exe")));
        assert_eq!(
            filename("yggdrasil/test.exe"),
            Some(String::from("test.exe"))
        );
        assert_eq!(
            filename("yggdrasil/test/visse"),
            Some(String::from("visse"))
        );
        assert_eq!(filename("."), None);
        assert_eq!(filename(".."), None);
    }

}
