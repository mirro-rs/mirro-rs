use std::path::{Path, PathBuf};

pub fn expand_tilde(input: impl AsRef<Path>) -> Option<PathBuf> {
    let path = input.as_ref();
    if !path.starts_with("~") {
        return Some(path.to_path_buf());
    }
    if path == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut to_return| {
        if to_return == Path::new("/") {
            // Already checked that we start with tilde
            path.strip_prefix("~").unwrap().to_path_buf()
        } else {
            to_return.push(path.strip_prefix("~/").unwrap());
            to_return
        }
    })
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn test_expand_tilde() {
    let home = std::env::var("HOME").unwrap();
    let projects = PathBuf::from(format!("{}/folder", home));
    assert_eq!(expand_tilde("~/folder"), Some(projects));
    assert_eq!(expand_tilde("/foo/bar"), Some("/foo/bar".into()));
    assert_eq!(
        expand_tilde("~alice/projects"),
        Some("~alice/projects".into())
    );
}
