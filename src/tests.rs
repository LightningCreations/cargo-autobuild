use std::{
    ffi::OsStr,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub fn which_any<S: AsRef<OsStr>>(names: &[S]) -> std::io::Result<PathBuf> {
    let paths = std::env::var("PATH").map_err(|e| std::io::Error::new(ErrorKind::NotFound, e))?;
    for i in std::env::split_paths(&paths) {
        for stem in names {
            let mut path = i.clone();
            path.push(Path::new(stem));
            let mut meta = std::fs::symlink_metadata(&path)?;
            while meta.file_type().is_symlink() {
                path = std::fs::read_link(path)?;
                meta = std::fs::symlink_metadata(&path)?;
            }
            if path.exists() {
                return Ok(path);
            }
        }
    }

    return Err(std::io::Error::new(
        ErrorKind::NotFound,
        "Cannot find files",
    ));
}

pub mod rustc {
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        io::{BufRead, ErrorKind},
        path::{Path, PathBuf},
        process::{Command, Stdio},
    };

    use target_tuples::{Target, Vendor};

    use crate::config::Step;

    #[derive(Default)]
    pub struct RustcTargetInfo {
        pub target: String,
        pub exe_suffix: OsString,
        pub rlib_prefix: OsString,
        pub rlib_suffix: OsString,
        pub dylib_prefix: OsString,
        pub dylib_suffix: OsString,
        pub staticlib_prefix: OsString,
        pub staticlib_suffix: OsString,
        pub cdylib_prefix: OsString,
        pub cdylib_suffix: OsString,
        pub procmacro_prefix: OsString,
        pub procmacro_suffix: OsString,
    }

    pub struct RustcTestsResult {
        pub rustc: PathBuf,
        pub rustflags: Vec<OsString>,
        pub no_std: bool,
        pub version: RustcVersion,
        pub target_info: RustcTargetInfo,
    }

    pub struct RustcVersion {
        pub prgname: String,
        pub major: i32,
        pub minor: i32,
        pub patch: i32,
        pub channel: RustcChannel,
    }

    pub enum RustcChannel {
        Stable,
        Beta,
        Nightly,
        Dev,
        Unstable,
    }

    fn find_rustc_target(
        rustc: &Path,
        flags: &mut String,
        file: &Path,
        target: &Target,
    ) -> std::io::Result<RustcTargetInfo> {
        let mut ret = RustcTargetInfo::default();

        if rustc
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(target.get_name())
        {
            match Command::new(rustc)
                .args(flags.split(' '))
                .arg("--crate-name")
                .arg("comptest")
                .arg("--crate-type")
                .arg("bin,rlib,dylib,staticlib,cdylib,proc-macro")
                .arg("--print")
                .arg("file-names")
                .arg(file)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => {
                    ret.target = target.get_name().to_string();
                    let mut lines = output.stdout.lines();

                    let exename = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;

                    ret.exe_suffix = exename
                        .find(".")
                        .map(|u| exename[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let rlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.rlib_prefix = rlibname
                        .find("comptest")
                        .map(|u| rlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.rlib_suffix = rlibname
                        .find(".")
                        .map(|u| rlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let dylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.dylib_prefix = dylibname
                        .find("comptest")
                        .map(|u| dylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.dylib_suffix = dylibname
                        .find(".")
                        .map(|u| dylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let staticlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.staticlib_prefix = staticlibname
                        .find("comptest")
                        .map(|u| staticlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.staticlib_suffix = staticlibname
                        .find(".")
                        .map(|u| staticlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let cdylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.cdylib_prefix = cdylibname
                        .find("comptest")
                        .map(|u| cdylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.cdylib_suffix = cdylibname
                        .find(".")
                        .map(|u| cdylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let procmacroname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.procmacro_prefix = procmacroname
                        .find("comptest")
                        .map(|u| procmacroname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.procmacro_suffix = procmacroname
                        .find(".")
                        .map(|u| procmacroname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    Ok(ret)
                }
                Ok(_) => Err(std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot execute {}", rustc.display()),
                )),
                Err(e) => Err(e),
            }
        } else {
            match Command::new(rustc)
                .args(flags.split(' '))
                .arg("--crate-name")
                .arg("comptest")
                .arg("--crate-type")
                .arg("bin,rlib,dylib,staticlib,cdylib,proc-macro")
                .arg("--target")
                .arg(target.get_name())
                .arg("--print")
                .arg("file-names")
                .arg(file)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => {
                    ret.target = target.get_name().to_string();
                    *flags += " --target ";
                    *flags += &ret.target;
                    let mut lines = output.stdout.lines();

                    let exename = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;

                    ret.exe_suffix = exename
                        .find(".")
                        .map(|u| exename[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let rlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.rlib_prefix = rlibname
                        .find("comptest")
                        .map(|u| rlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.rlib_suffix = rlibname
                        .find(".")
                        .map(|u| rlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let dylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.dylib_prefix = dylibname
                        .find("comptest")
                        .map(|u| dylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.dylib_suffix = dylibname
                        .find(".")
                        .map(|u| dylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let staticlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.staticlib_prefix = staticlibname
                        .find("comptest")
                        .map(|u| staticlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.staticlib_suffix = staticlibname
                        .find(".")
                        .map(|u| staticlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let cdylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.cdylib_prefix = cdylibname
                        .find("comptest")
                        .map(|u| cdylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.cdylib_suffix = cdylibname
                        .find(".")
                        .map(|u| cdylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let procmacroname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.procmacro_prefix = procmacroname
                        .find("comptest")
                        .map(|u| procmacroname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.procmacro_suffix = procmacroname
                        .find(".")
                        .map(|u| procmacroname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    return Ok(ret);
                }
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            match Command::new(rustc)
                .args(flags.split(' '))
                .arg("--crate-name")
                .arg("comptest")
                .arg("--crate-type")
                .arg("bin,rlib,dylib,staticlib,cdylib,proc-macro")
                .arg("--target")
                .arg(target.to_string())
                .arg("--print")
                .arg("file-names")
                .arg(file)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => {
                    ret.target = target.to_string();
                    *flags += " --target ";
                    *flags += &ret.target;
                    let mut lines = output.stdout.lines();

                    let exename = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;

                    ret.exe_suffix = exename
                        .find(".")
                        .map(|u| exename[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let rlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.rlib_prefix = rlibname
                        .find("comptest")
                        .map(|u| rlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.rlib_suffix = rlibname
                        .find(".")
                        .map(|u| rlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let dylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.dylib_prefix = dylibname
                        .find("comptest")
                        .map(|u| dylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.dylib_suffix = dylibname
                        .find(".")
                        .map(|u| dylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let staticlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.staticlib_prefix = staticlibname
                        .find("comptest")
                        .map(|u| staticlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.staticlib_suffix = staticlibname
                        .find(".")
                        .map(|u| staticlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let cdylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.cdylib_prefix = cdylibname
                        .find("comptest")
                        .map(|u| cdylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.cdylib_suffix = cdylibname
                        .find(".")
                        .map(|u| cdylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let procmacroname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.procmacro_prefix = procmacroname
                        .find("comptest")
                        .map(|u| procmacroname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.procmacro_suffix = procmacroname
                        .find(".")
                        .map(|u| procmacroname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    return Ok(ret);
                }
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            let ntarget = Target::from_components(
                target.arch(),
                Vendor::Unknown,
                target.operating_system(),
                target.environment(),
                target.object_format(),
            );

            match Command::new(rustc)
                .args(flags.split(' '))
                .arg("--crate-name")
                .arg("comptest")
                .arg("--crate-type")
                .arg("bin,rlib,dylib,staticlib,cdylib,proc-macro")
                .arg("--target")
                .arg(target.to_string())
                .arg("--print")
                .arg("file-names")
                .arg(file)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => {
                    ret.target = ntarget.to_string();
                    *flags += " --target ";
                    *flags += &ret.target;
                    let mut lines = output.stdout.lines();

                    let exename = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;

                    ret.exe_suffix = exename
                        .find(".")
                        .map(|u| exename[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let rlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.rlib_prefix = rlibname
                        .find("comptest")
                        .map(|u| rlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.rlib_suffix = rlibname
                        .find(".")
                        .map(|u| rlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let dylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.dylib_prefix = dylibname
                        .find("comptest")
                        .map(|u| dylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.dylib_suffix = dylibname
                        .find(".")
                        .map(|u| dylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let staticlibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.staticlib_prefix = staticlibname
                        .find("comptest")
                        .map(|u| staticlibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.staticlib_suffix = staticlibname
                        .find(".")
                        .map(|u| staticlibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let cdylibname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.cdylib_prefix = cdylibname
                        .find("comptest")
                        .map(|u| cdylibname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.cdylib_suffix = cdylibname
                        .find(".")
                        .map(|u| cdylibname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    let procmacroname = lines.next().ok_or_else(|| {
                        std::io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Could not determine file names from invoking {}",
                                rustc.display()
                            ),
                        )
                    })??;
                    ret.procmacro_prefix = procmacroname
                        .find("comptest")
                        .map(|u| procmacroname[..u].to_string())
                        .unwrap_or_default()
                        .into();

                    ret.procmacro_suffix = procmacroname
                        .find(".")
                        .map(|u| procmacroname[u..].to_string())
                        .unwrap_or_default()
                        .into();

                    return Ok(ret);
                }
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            Err(std::io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Could not determine how to compile for {} using {}",
                    target.get_name(),
                    rustc.display()
                ),
            ))
        }
    }

    pub fn find_compiler(
        var: &OsStr,
        flags_var: &OsStr,
        target: &Target,
        cross_compiling: bool,
        tmpdir: &Path,
    ) -> std::io::Result<RustcTestsResult> {
        let mut flags = match std::env::var(flags_var) {
            Ok(flags) => flags,
            Err(std::env::VarError::NotPresent) => "-O -g".to_string(),
            Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, e)),
        };

        let rustc = if let Some(path) = std::env::var_os(var) {
            PathBuf::from(path)
        } else {
            super::which_any(&[
                "rustc",
                "lcrustc",
                &format!("{}-gccrs", target.get_name()),
                "gccrs",
            ])?
        };

        let comptest_path = {
            let mut path = tmpdir.to_owned();
            path.push("comptest.rs");
            path
        };

        std::fs::write(
            &comptest_path,
            r#"
fn main(){}

"#,
        )?;

        let targ = find_rustc_target(&rustc, &mut flags, &comptest_path, &target)?;
        let out = Command::new(&rustc).arg("--version").output()?;

        let version = out.stdout.lines().next().ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::Unsupported,
                format!("Cannot determine the version of {}", rustc.display()),
            )
        })??;

        let mut components = version.split(' ');
        let name = components.next().ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::Unsupported,
                format!("Cannot determine the version of {}", rustc.display()),
            )
        })?;
        let mut prgname = None;
        if name != "rustc" {
            prgname = Some(name);
        }

        let ver = components.next().ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::Unsupported,
                format!("Cannot determine the version of {}", rustc.display()),
            )
        })?;

        let mut parts = ver.split(".");
        let major = parts
            .next()
            .ok_or_else(|| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?
            .parse()
            .map_err(|_| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?;
        let minor = parts
            .next()
            .ok_or_else(|| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?
            .parse()
            .map_err(|_| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?;
        let tail = parts.next().ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::Unsupported,
                format!("Cannot determine the version of {}", rustc.display()),
            )
        })?;

        let mut patch_and_maybe_channel = tail.split('-');
        let patch = patch_and_maybe_channel
            .next()
            .ok_or_else(|| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?
            .parse()
            .map_err(|_| {
                std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!("Cannot determine the version of {}", rustc.display()),
                )
            })?;

        let mut channel = if let Some(channel) = patch_and_maybe_channel.next() {
            match channel {
                "beta" => RustcChannel::Beta,
                "nightly" => RustcChannel::Nightly,
                _ => RustcChannel::Dev,
            }
        } else {
            RustcChannel::Stable
        };

        if let Some(paren) = components.next() {
            let paren = paren.get(1..);
            match paren {
                Some("mrustc") => {
                    prgname = Some("mrust");
                }
                Some("lccc") => {
                    prgname = Some("lcrustc");
                }
                _ => {}
            }
        }

        let prgname = match prgname {
            Some(s) => s,
            None => "rustc",
        }
        .to_string();

        if prgname.starts_with("lc") {
            channel = RustcChannel::Unstable
        }

        let version = RustcVersion {
            major,
            minor,
            patch,
            prgname,
            channel,
        };

        let output_file = {
            let mut path = tmpdir.to_owned();
            let mut name = OsString::from("comptest");
            name.push(&targ.exe_suffix);
            path.push(name);
            path
        };

        if Command::new(&rustc)
            .args(flags.split(' '))
            .arg("--crate-type")
            .arg("bin")
            .arg("--emit")
            .arg(format!("link={}", output_file.display()))
            .arg("--crate-name")
            .arg("comptest")
            .arg(&comptest_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .status()?
            .success()
        {
            if !cross_compiling {
                if !Command::new(&output_file).status()?.success() {
                    return Err(std::io::Error::new(
                        ErrorKind::Unsupported,
                        format!("Cannot execute binaries produced by {}", rustc.display()),
                    ));
                }
            }
            return Ok(RustcTestsResult {
                rustc,
                rustflags: flags.split(" ").map(|s| OsString::from(s)).collect(),
                no_std: false,
                version,
                target_info: targ,
            });
        };

        std::fs::write(
            &comptest_path,
            r#"
#![no_std]
"#,
        )?;

        let output_file = {
            let mut path = tmpdir.to_owned();
            let mut name = OsString::from(&targ.rlib_prefix);
            name.push("comptest");
            name.push(&targ.rlib_suffix);
            path.push(name);
            path
        };

        if Command::new(&rustc)
            .args(flags.split(' '))
            .arg("--crate-type")
            .arg("rlib")
            .arg("--emit")
            .arg(format!("link={}", output_file.display()))
            .arg("--crate-name")
            .arg("comptest")
            .arg(&comptest_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .status()?
            .success()
        {
            Ok(RustcTestsResult {
                rustc,
                rustflags: flags.split(" ").map(|s| OsString::from(s)).collect(),
                no_std: false,
                version,
                target_info: targ,
            })
        } else {
            Err(std::io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Cannot compile simple test program with {}",
                    rustc.display()
                ),
            ))
        }
    }
}
