use std::path::{Path, PathBuf};

mod error;

pub use error::*;

pub fn join<T: AsRef<Path>, S: AsRef<str>>(base: T, cmp: S) -> Result<PathBuf> {
    let mut path = cmp.as_ref();
    let mut base = base.as_ref();
    if path.starts_with("./") {
        path = path.trim_left_matches("./");
    } else if path == "." {
        return Ok(base.to_path_buf());
    }

    while path.starts_with("..") {
        base = match base.parent() {
            Some(parent) => parent,
            None => return Err(Error {}),
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
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

}
