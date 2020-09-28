use super::error::{Error, ErrorKind, Result};

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
        base = match parent_path(&base) {
            Some(p) => p.to_owned(),
            None => return Err(Error::new(ErrorKind::Unknown)),
        };
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

// pub fn parent_path<S: AsRef<str>>(input: S) -> String {
//     let input = input.as_ref();
//     if input.is_empty() {
//         return ".".to_owned();
//     }

//     let sep = '/';

//     let code = input.chars().nth(0).unwrap();
//     let has_root = code == sep;

//     let mut end: i32 = -1;
//     let mut matched = true;
//     let len = input.len();
//     for (i, c) in input.chars().rev().enumerate() {
//         if c == sep {
//             if !matched {
//                 end = (len - i) as i32;
//                 break;
//             }
//         } else {
//             matched = false;
//         }
//     }

//     if end == -1 {
//         return if has_root {
//             "/".to_owned()
//         } else {
//             ".".to_owned()
//         };
//     } else if has_root && end == 1 {
//         return "//".to_string();
//     }

//     input.chars().take((end - 1) as usize).collect()
// }

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

pub fn extname(filename: &str) -> Option<&str> {
    if filename == "." || filename == ".." {
        return None;
    }
    let len = filename.len();
    let mut index = -1;
    'outer: for i in (0..len).rev() {
        match filename.chars().nth(i) {
            Some('/') => break 'outer,
            Some('.') => {
                if i == 0 {
                    break 'outer;
                } else if filename.chars().nth(i - 1).unwrap() == '/' {
                    break 'outer;
                }
                index = i as i32;
                break 'outer;
            }
            _ => {}
        }
    }

    if index == -1 {
        return None;
    }

    Some(&filename[filename.char_indices().nth(index as usize).unwrap().0..filename.len()])
    // Some(&filename.chars().skip(index as usize).collect())
}

pub fn set_extname<T: AsRef<str>, S: AsRef<str>>(filename: T, ext: S) -> String {
    let e = extname(filename.as_ref());
    let ext = match ext.as_ref().chars().nth(0) {
        Some(m) => {
            if m == '.' {
                ext.as_ref().trim_start_matches(".")
            } else {
                ext.as_ref()
            }
        }
        None => return filename.as_ref().to_string(),
    };

    match e {
        None => format!("{}.{}", filename.as_ref(), ext),
        Some(m) => filename
            .as_ref()
            .replace(&m, &format!(".{}", ext))
            .to_string(),
    }
}

pub fn parent_path(filename: &str) -> Option<&str> {
    if filename.is_empty() || filename == "/" || filename == "." || filename == ".." {
        return None;
    }

    let mut r = filename;
    if r.chars().last().unwrap() == '/' {
        r = r.trim_end_matches("/");
    }

    // match r.rfind('/') {
    //     Some(idx) => Some(r.chars().take(idx + 1).collect()),
    //     None => Some("".to_owned()),
    // }

    match r.rfind('/') {
        Some(idx) => Some(&r[0..r.char_indices().nth(idx + 1).unwrap().0]),
        None => None,
    }
}

pub fn filename(path: &str) -> Option<&str> {
    if path.is_empty()
        || path == "/"
        || path == "."
        || path == ".."
        || path.chars().last() == Some('/')
    {
        return None;
    }

    match path.rfind('/') {
        Some(idx) => Some(&path[path.char_indices().nth(idx + 1).unwrap().0..path.len()]),
        None => Some(path),
    }
}

pub fn set_filename(path: &str, filename: &str) -> String {
    if path.chars().last() == Some('/') {
        join(path, filename)
    } else {
        match parent_path(path) {
            Some(path) => join(path, filename),
            None => filename.to_owned(),
        }
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
        assert_eq!(extname("test.exe"), Some(".exe"));
        assert_eq!(extname("yggdrasil/test.exe"), Some(".exe"));
        assert_eq!(extname("yggdrasil/test"), None);
        assert_eq!(extname("yggdrasil/.test"), None);
        assert_eq!(extname(".yggdrasil/test"), None);
        assert_eq!(extname("./yggdrasil/test"), None);
        assert_eq!(extname("."), None);
        assert_eq!(extname(".."), None);
    }

    #[test]
    fn setextname_test() {
        assert_eq!(set_extname("test", "exe"), String::from("test.exe"));
        assert_eq!(
            set_extname("yggdrasil/test.exe", "txt"),
            String::from("yggdrasil/test.txt")
        );
        assert_eq!(
            set_extname("yggdrasil/test", ".md"),
            String::from("yggdrasil/test.md")
        );
        // assert_eq!(extname("."), None);
        // assert_eq!(extname(".."), None);
    }

    #[test]
    fn parent_test() {
        assert_eq!(parent_path("test.exe"), None);
        assert_eq!(parent_path("yggdrasil/test.exe"), Some("yggdrasil/"));
        assert_eq!(parent_path("yggdrasil/test/visse"), Some("yggdrasil/test/"));
        assert_eq!(parent_path("."), None);
        assert_eq!(parent_path(".."), None);
    }

    #[test]
    fn test_resolve() {
        assert_eq!(resolve("/test", "../").unwrap().as_str(), "/");
        assert_eq!(resolve("/test/rapper", "../").unwrap().as_str(), "/test/");
        assert_eq!(resolve("/test/rapper", "../../").unwrap().as_str(), "/");
        assert!(resolve("/test", "../../").is_err())
    }

    // #[test]
    // fn parent_path_test() {
    //     assert_eq!(parent_path("test.exe"), String::from("."));
    //     assert_eq!(parent_path("yggdrasil/test.exe"), String::from("yggdrasil"));
    //     assert_eq!(
    //         parent_path("yggdrasil/test/visse"),
    //         String::from("yggdrasil/test")
    //     );

    //     // assert_eq!(parent_path("."), None);
    //     // assert_eq!(parent_path(".."), None);
    // }

    #[test]
    fn filename_test() {
        assert_eq!(filename("test.exe"), Some("test.exe"));
        assert_eq!(filename("yggdrasil/test.exe"), Some("test.exe"));
        assert_eq!(filename("yggdrasil/test/visse"), Some("visse"));
        assert_eq!(filename("yggdrasil/test/visse/"), None);
        assert_eq!(filename("."), None);
        assert_eq!(filename(".."), None);
    }

    #[test]
    fn set_filename_test() {
        assert_eq!(set_filename("test/", "test.exe"), "test/test.exe");
        assert_eq!(set_filename("test", "test.exe"), "test.exe");
        assert_eq!(set_filename("test/free", "test.exe"), "test/test.exe");
        assert_eq!(set_filename("test/free/", "test.exe"), "test/free/test.exe");

        assert_eq!(set_filename("", "test.exe"), "test.exe");
    }
}
