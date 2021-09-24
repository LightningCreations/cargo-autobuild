use std::{collections::HashMap, ffi::OsString, path::PathBuf, str::FromStr};

use serde::{Deserialize, Deserializer};
use target_tuples::Target;

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum InstallDirectory {
    Prefix,
    ExecPrefix,
    BinDir,
    SbinDir,
    LibexecDir,
    LibDir,
    IncludeDir,
    DatarootDir,
    DataDir,
    DocDir,
    InfoDir,
    ManDir,
    HtmlDir,
    PdfDir,
    DviDir,
    LocaleDir,
    LocalStateDir,
    SharedStateDir,
    RunStateDir,
    SysconfDir,
    Custom(String),
}

fn parse_target<'de, D: Deserializer<'de>>(de: D) -> Result<Target, D::Error> {
    let st = <&str>::deserialize(de)?;
    Target::from_str(st)
        .map_err(|_| <D::Error as serde::de::Error>::custom(format_args!("Unknown target {}", st)))
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum BuildTarget {
    #[serde(rename = "$build")]
    Build,
    #[serde(rename = "$host")]
    Host,
    #[serde(rename = "$target")]
    Target,
    Input(#[serde(deserialize_with = "parse_target")] Target),
}

fn host() -> BuildTarget {
    BuildTarget::Host
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum Step {
    BuildCrate(BuildCrateStep),
    Subdirectory(SubdirectoryStep),
    Install(InstallStep),
    Command(CommandStep),
    GenerateDocs(GenerateDocsStep),
    ConfigureFile(ConfigureFileStep),
}

pub enum InstallTarget {
    Base {
        base: InstallDirectory,
        path: PathBuf,
    },
    Absolute(PathBuf),
}

impl<'de> Deserialize<'de> for InstallTarget {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let st = <&str>::deserialize(de)?;
        if st.starts_with("$") {
            let mut parts = st[1..].split('/');
            let dir = parts.next().unwrap();
            let path = parts.collect();
            let base = match dir {
                "prefix" => InstallDirectory::Prefix,
                "exec-prefix" => InstallDirectory::ExecPrefix,
                "bindir" => InstallDirectory::BinDir,
                "sbindir" => InstallDirectory::SbinDir,
                "libdir" => InstallDirectory::LibDir,
                "libexecdir" => InstallDirectory::LibexecDir,
                "includedir" => InstallDirectory::IncludeDir,
                "datadir" => InstallDirectory::DataDir,
                "datarootdir" => InstallDirectory::DatarootDir,
                "docdir" => InstallDirectory::DocDir,
                "infodir" => InstallDirectory::InfoDir,
                "mandir" => InstallDirectory::ManDir,
                "htmldir" => InstallDirectory::HtmlDir,
                "pdfdir" => InstallDirectory::PdfDir,
                "dvidir" => InstallDirectory::DviDir,
                "localedir" => InstallDirectory::LocaleDir,
                "localstatedir" => InstallDirectory::LocalStateDir,
                "sharedstatedir" => InstallDirectory::SharedStateDir,
                "runstatedir" => InstallDirectory::RunStateDir,
                "sysconfdir" => InstallDirectory::SysconfDir,
                x => InstallDirectory::Custom(x.to_string()),
            };
            Ok(Self::Base { base, path })
        } else {
            Ok(InstallTarget::Absolute(PathBuf::from(st)))
        }
    }
}

#[derive(Deserialize)]
pub struct Directories {
    #[serde(flatten)]
    pub dirs: HashMap<String, InstallTarget>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]

pub struct BuildCrateStep {
    #[serde(default)]
    pub path: OsString,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default = "host")]
    pub target: BuildTarget,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GenerateDocsStep {}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubdirectoryStep {}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Mode {
    Octal(i32),
    Chmod(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InstallStep {
    pub file: PathBuf,
    pub target: InstallTarget,
    #[serde(default)]
    pub mode: Option<Mode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommandStep {
    pub cmd: PathBuf,
    #[serde(default)]
    pub args: Vec<OsString>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigureFileStep {
    pub base: PathBuf,
    #[serde(default)]
    pub input: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
pub enum ProgramType {
    Rustc,
    Cargo,
    Cc,
    Cxx,
    As,
    Ar,
    Ld,
    Objdump,
    Objcopy,
    Strip,
    Ln,
    LnS,
    Install,
    Yacc,
    Lex,
    Other(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Program {
    #[serde(rename = "type")]
    pub ty: ProgramType,
    #[serde(default)]
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub test_steps: Vec<Step>,
    #[serde(default)]
    pub compiler_target: Option<BuildTarget>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}
