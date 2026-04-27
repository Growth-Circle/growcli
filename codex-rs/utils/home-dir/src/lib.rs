use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

/// Returns the path to the CLI configuration directory.
///
/// For the `codex` executable, this can be specified by the `CODEX_HOME`
/// environment variable. If not set, it defaults to `~/.codex`.
///
/// For the `growcli` executable, this can be specified by the `GROWCLI_HOME`
/// environment variable, then by `CODEX_HOME`. If neither is set, it defaults
/// to `~/.growcli` so Grow CLI state does not affect local Codex installs.
///
/// - If an override environment variable is set, the value must exist and be a
///   directory. The value will be canonicalized and this function will Err
///   otherwise.
/// - If no override environment variable is set, this function does not verify
///   that the directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let growcli_home_env = std::env::var("GROWCLI_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(
        growcli_home_env.as_deref(),
        codex_home_env.as_deref(),
        std::env::args_os().next().as_deref(),
    )
}

fn find_codex_home_from_env(
    growcli_home_env: Option<&str>,
    codex_home_env: Option<&str>,
    argv0: Option<&std::ffi::OsStr>,
) -> std::io::Result<AbsolutePathBuf> {
    let is_growcli = argv0
        .and_then(|arg| std::path::Path::new(arg).file_stem())
        .is_some_and(|stem| stem == "growcli");

    let override_env = if is_growcli {
        growcli_home_env
            .map(|val| ("GROWCLI_HOME", val))
            .or_else(|| codex_home_env.map(|val| ("CODEX_HOME", val)))
    } else {
        codex_home_env.map(|val| ("CODEX_HOME", val))
    };

    match override_env {
        Some((env_name, val)) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{env_name} points to {val:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read {env_name} {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{env_name} points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize {env_name} {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(if is_growcli { ".growcli" } else { ".codex" });
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            Some(missing_str),
            Some("codex".as_ref()),
        )
        .expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            Some(file_str),
            Some("codex".as_ref()),
        )
        .expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            Some(temp_str),
            Some("codex".as_ref()),
        )
        .expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let resolved = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            /*codex_home_env*/ None,
            Some("codex".as_ref()),
        )
        .expect("default CODEX_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".codex");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_growcli_default_for_growcli_binary() {
        let resolved = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            /*codex_home_env*/ None,
            Some("growcli".as_ref()),
        )
        .expect("default GROWCLI_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".growcli");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_prefers_growcli_home_for_growcli_binary() {
        let growcli_home = TempDir::new().expect("growcli home");
        let codex_home = TempDir::new().expect("codex home");

        let resolved = find_codex_home_from_env(
            growcli_home.path().to_str(),
            codex_home.path().to_str(),
            Some("growcli".as_ref()),
        )
        .expect("GROWCLI_HOME");
        let expected = growcli_home.path().canonicalize().expect("canonicalize");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_uses_codex_home_fallback_for_growcli_binary() {
        let codex_home = TempDir::new().expect("codex home");

        let resolved = find_codex_home_from_env(
            /*growcli_home_env*/ None,
            codex_home.path().to_str(),
            Some("growcli".as_ref()),
        )
        .expect("CODEX_HOME fallback");
        let expected = codex_home.path().canonicalize().expect("canonicalize");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }
}
