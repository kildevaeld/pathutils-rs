use super::error::{Error, Result, ErrorKind};
use regex::Regex;
use std::collections::HashMap;
use std::fs;


lazy_static::lazy_static! {
    static ref GLOB_STRICT_REGEX: Regex = {
        Regex::new(r"\\(.)|(^!|\*|[\].+)]\?|\[[^\\\]]+\]|\{[^\\}]+\}|\(\?[:!=][^\\)]+\)|\([^|]+\|[^\\)]+\))").unwrap()
    };

    static ref GLOB_REGEX: Regex = {
        Regex::new(r"\\(.)|(^!|[*?{}()[\]]|\(\?)").unwrap()
    };

    static ref GLOB_SPECIAL: Regex = {
        Regex::new(r"[\{\[].*[\\/]*.*[\}\]]$").unwrap()
    };

    static ref GLOB_GROUP: Regex = {
        Regex::new(r"(^|[^\\])([\{\[]|\([^\)]+$)").unwrap()
    };

    static ref GLOB_ESCAPE: Regex = {
        Regex::new(r"\\([\*\?\|\[\]\(\)\{\}])").unwrap()
    };

    static ref PAIRS: HashMap<char, char> = {
        let mut m = HashMap::new();
        m.insert('{', '}');
        m.insert('(', ')');
        m.insert('[', ']');
        m
    };

}

pub fn is_absolute<S: AsRef<str>>(input: S) -> bool {
    let re = input.as_ref();
    re.len() > 0 && re.chars().nth(0).unwrap() == '/'
}

pub struct Negation<'a> {
    pub negated: bool,
    pub pattern: &'a str,
    pub original: &'a str,
}

pub fn is_negated_glob<'a>(input: &'a str) -> bool {
    if input.len() == 0 {
        panic!("input cannot be empty");
    }

    if input.len() >= 2
        && input.chars().nth(0).unwrap() == '!'
        && input.chars().nth(1).unwrap() != '('
    {
        return true;
        //out.pattern =
    }

    false
}

pub fn join(root: &str, glob: &str) -> String {
    let l = root.len();
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

pub fn glob_parent<S: AsRef<str>>(glob: S) -> String {
    let mut output = glob.as_ref().to_owned();

    if GLOB_SPECIAL.is_match(&output) {
        output.push('/');
    }

    output.push('a');

    loop {
        output = parent_path(output);
        let len = output.len();
        if is_glob(&output, true) || GLOB_GROUP.is_match(&output) {
            continue;
        } else if len >= 2
            && output.chars().nth(len - 1).unwrap() == '.'
            && output.chars().nth(len - 2).unwrap() == '.'
        {
            output = parent_path(output);
        } else {
            break;
        }
    }

    output
}

pub fn to_absolute_glob<S: AsRef<str>>(input: S, cwd: S) -> Result<String> {
    let cwd = if is_absolute(cwd.as_ref()) {
        cwd.as_ref().to_owned()
    } else {
        let can = fs::canonicalize(cwd.as_ref())?;
        match can.to_str() {
            Some(o) => o.to_owned(),
            None => return Err(Error::new(ErrorKind::Unknown)),
        }
    };

    let mut output = input.as_ref().to_owned();

    if output.len() >= 2
        && output.chars().nth(0).unwrap() == '.'
        && output.chars().nth(1).unwrap() == '/'
    {
        output.remove(0);
        output.remove(0);
    }

    if output.len() == 1 && output.chars().nth(0).unwrap() == '.' {
        output.clear();
        return Ok(output);
    }

    if !is_absolute(&output)
        || (output.len() >= 2
            && output.chars().nth(0).unwrap() == '\\'
            && output.chars().nth(0).unwrap() == '\\')
    {
        output = join(&cwd, &output);
    }

    Ok(output)
}

pub fn is_glob<S: AsRef<str>>(input: S, strict: bool) -> bool {
    let s = input.as_ref();

    let mut idx = 0;
    loop {
        let slice = s.chars().skip(idx).collect::<String>();

        let m = match if strict {
            GLOB_STRICT_REGEX.captures(&slice)
        } else {
            GLOB_REGEX.captures(&slice)
        } {
            Some(m) => m,
            None => return false,
        };

        if m.len() > 2 {
            return true;
        }

        let full = m.get(0).unwrap().as_str();
        let mut found = false;
        if let Some(v) = PAIRS.get(&full.chars().nth(0).unwrap()) {
            if let Some(i) = full.find(|m| m == *v) {
                idx = i + 1;
                found = true;
            }
        }

        if !found {
            idx = m.get(0).unwrap().end();
        }
    }
}

pub fn to_absolute<S: AsRef<str>, C: AsRef<str>>(path: S, cwd: C) -> String {
    let mut path = path.as_ref().to_string();

    if !is_absolute(&path) {
        path = join(cwd.as_ref(), &path);
    }

    path
}

#[cfg(test)]
pub mod tests {

    use super::is_glob;

    #[test]
    fn valid_is_globs() {
        assert_eq!(is_glob("!foo.js", true), true);
        assert_eq!(is_glob("*.js", true), true);
        assert_eq!(is_glob("**/abc.js", true), true);
        assert_eq!(is_glob("abc/*.js", true), true);
        assert_eq!(is_glob("abc/(aaa|bbb).js", true), true);
        assert_eq!(is_glob("abc/[a-z].js", true), true);
        assert_eq!(is_glob("abc/{a,b}.js", true), true);
    }

    #[test]
    fn invalid_is_globs() {
        assert_eq!(is_glob("abc.js", true), false);
        assert_eq!(is_glob("abc/def/ghi.js", true), false);
        assert_eq!(is_glob("foo.js", true), false);
        assert_eq!(is_glob("abc/@.js", true), false);
        assert_eq!(is_glob("abc/+.js", true), false);
        assert_eq!(is_glob("abc/?.js", true), false);
    }
}