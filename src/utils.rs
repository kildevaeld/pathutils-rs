use super::error::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::str::Chars;

pub fn resolve<T: AsRef<str>, S: AsRef<str>>(base: T, cmp: S) -> Result<String> {
    let mut path = cmp.as_ref();
    let mut base = base.as_ref().to_string();
    if path.starts_with("./") {
        path = path.trim_start_matches("./");
    } else if path == "." {
        return Ok(base.to_string());
    } else if path.starts_with("/") {
        path = path.trim_start_matches("/");
    }

    while path.starts_with("..") {
        base = match parent(base) {
            Some(p) => p,
            None => return Err(Error::new(ErrorKind::Unknown)),
        };
        //base = parent_path(path);
        path = path.trim_start_matches("..");

        if !path.is_empty() && path.chars().nth(0).unwrap() == '/' {
            path = path.trim_start_matches("/");
        }
    }

    Ok(join(&base, path))
}

// pub fn join<T: AsRef<Path>, S: AsRef<str>>(base: T, cmp: S) -> Result<PathBuf> {
//     let mut path = cmp.as_ref();
//     let mut base = base.as_ref();
//     if path.starts_with("./") {
//         path = path.trim_start_matches("./");
//     } else if path == "." {
//         return Ok(base.to_path_buf());
//     } else if path.starts_with("/") {
//         path = path.trim_start_matches("/");
//     }

//     while path.starts_with("..") {
//         base = match base.parent() {
//             Some(parent) => parent,
//             None => return Err(Error::new(ErrorKind::Unknown)),
//         };
//         path = path.trim_start_matches("..");
//         if !path.is_empty() && path.chars().nth(0).unwrap() == '/' {
//             path = path.trim_start_matches("/");
//         }
//     }

//     Ok(base.join(path))
// }

pub fn join(root: &str, glob: &str) -> String {
    let l = root.len();
    if glob.is_empty() {
        return root.to_string();
    } else if root.is_empty() {
        return glob.to_string();
    }
    if root.chars().nth(l - 1).unwrap() == '/' && glob.chars().nth(0).unwrap() == '/' {
        let mut root: String = root.chars().take(l - 1).collect();
        root.reserve_exact(glob.len());
        root.insert_str(l - 1, glob);
        return root;
    } else if root.chars().nth(l - 1).unwrap() == '/' || glob.chars().nth(0).unwrap() == '/' {
        let mut out = String::with_capacity(root.len() + glob.len());
        out.insert_str(0, root);
        out.insert_str(root.len(), glob);
        return out;
    } else {
        let mut out = String::with_capacity(root.len() + glob.len() + 1);
        out.insert_str(0, root);
        out.insert(root.len(), '/');
        out.insert_str(root.len() + 1, glob);
        return out;
    }
}

pub fn parent_path<S: AsRef<str>>(input: S) -> String {
    let input = input.as_ref();
    if input.is_empty() {
        return ".".to_owned();
    }

    let sep = '/';

    let code = input.chars().nth(0).unwrap();
    let has_root = code == sep;

    let mut end: i32 = -1;
    let mut matched = true;
    let len = input.len();
    for (i, c) in input.chars().rev().enumerate() {
        if c == sep {
            if !matched {
                end = (len - i) as i32;
                break;
            }
        } else {
            matched = false;
        }
    }

    if end == -1 {
        return if has_root {
            "/".to_owned()
        } else {
            ".".to_owned()
        };
    } else if has_root && end == 1 {
        return "//".to_string();
    }

    input.chars().take((end - 1) as usize).collect()
}

pub fn join_slice<T: AsRef<str>, S: AsRef<str>>(base: T, cmps: &[S]) -> Result<String> {
    let mut path = base.as_ref().to_string();
    for p in cmps {
        path = resolve(&path, p.as_ref())?;
    }
    Ok(path)
}

pub fn is_absolute<S: AsRef<str>>(input: S) -> bool {
    let re = input.as_ref();
    re.len() > 0 && re.chars().nth(0).unwrap() == '/'
}

pub fn to_absolute<S: AsRef<str>, C: AsRef<str>>(path: S, cwd: C) -> Result<String> {
    let mut path = path.as_ref().to_string();

    if !is_absolute(&path) {
        path = resolve(cwd.as_ref(), &path)?;
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

pub fn set_extname<T: AsRef<str>, S: AsRef<str>>(filename: T, ext: S) -> String {
    let e = extname(filename.as_ref());
    match e {
        None => format!("{}.{}", filename.as_ref(), ext.as_ref()),
        Some(m) => filename.as_ref().replace(&m, ext.as_ref()).to_string(),
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
        let path = "/test"; // Path::new("/test");
        assert_eq!(
            resolve(path, "test/me.txt").expect("join"),
            String::from("/test/test/me.txt")
        );

        assert_eq!(
            resolve(path, "../me.txt").expect("parent"),
            String::from("/me.txt")
        );
        assert_eq!(resolve(path, ".").expect("this"), String::from("/test"));
        assert_eq!(resolve(path, "..").expect("parent"), String::from("/"));
        assert!(resolve(path, "../../me.txt").is_err());
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
    fn set_extname_test() {
        assert_eq!(set_extname("test", "exe"), String::from("test.exe"));
        assert_eq!(
            set_extname("yggdrasil/test.exe", "txt"),
            String::from("yggdrasil/test.txt")
        );
        assert_eq!(
            set_extname("yggdrasil/test", "md"),
            String::from("yggdrasil/test.md")
        );
        // assert_eq!(extname("."), None);
        // assert_eq!(extname(".."), None);
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
    fn parent_path_test() {
        assert_eq!(parent_path("test.exe"), String::from("."));
        assert_eq!(parent_path("yggdrasil/test.exe"), String::from("yggdrasil"));
        assert_eq!(
            parent_path("yggdrasil/test/visse"),
            String::from("yggdrasil/test")
        );

        // assert_eq!(parent_path("."), None);
        // assert_eq!(parent_path(".."), None);
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
