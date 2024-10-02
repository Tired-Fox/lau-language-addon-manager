//! Rust representation of the lua language server configuration file.
//!
//! Check the official documentation for possible settings and values.
//! - https://luals.github.io/wiki/settings/#settings
//!
//! This library will add custom fields to different settings/configurations. This allows
//! it to use the `.luarc.json` file alongside luals.
//!
//! # Added Fields:
//! - `workspace.addons`: An object of where the key is the addon name and the value is a json
//! representation of [`Addon`][crate::Addon]. This information is used to know what addons are
//! currently installed. Similar to the `"dependencies"` entry in a `npm` project's `package.json`

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;

use crate::{
    diagnostics::{Diagnostic, DiagnosticGroup},
    Addon, Error, ADDONS_DIR,
};

const fn enabled(ctx: &bool) -> bool {
    *ctx
}

const fn disabled(ctx: &bool) -> bool {
    !*ctx
}

const fn zero(ctx: &usize) -> bool {
    *ctx == 0
}

#[inline]
const fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddonManager {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for AddonManager {
    fn default() -> Self {
        Self {
            enable: true,
            other: None,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Snippet {
    #[default]
    Disable,
    Replace,
    Both,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Show {
    Disable,
    Enable,
    #[default]
    Fallback,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub auto_require: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_snippet: Option<Snippet>,
    #[serde(default, skip_serializing_if = "zero")]
    pub display_context: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_snippet: Option<Snippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postfix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_separator: Option<String>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub show_params: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_word: Option<Show>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub workspace_word: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for Completion {
    fn default() -> Self {
        Self {
            enable: true,
            auto_require: true,
            call_snippet: None,
            display_context: 0,
            keyword_snippet: None,
            postfix: None,
            require_separator: None,
            show_params: true,
            show_word: None,
            workspace_word: true,

            other: None,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum FileState {
    Any,
    Opened,
    None,
    #[default]
    Fallback,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum GroupSeverity {
    Error,
    Warning,
    Information,
    Hint,
    #[default]
    Fallback,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
    #[serde(rename = "Error!")]
    ErrorBang,
    #[serde(rename = "Warning!")]
    WarningBang,
    #[serde(rename = "Information!")]
    InformationBang,
    #[serde(rename = "Hint!")]
    HintBang,
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "information" => Self::Information,
            "hint" => Self::Hint,
            "error!" => Self::ErrorBang,
            "warning!" => Self::WarningBang,
            "information!" => Self::InformationBang,
            "hint!" => Self::HintBang,
            other => return Err(format!("invalid diagnostic severity: {other}")),
        })
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Files {
    Enable,
    #[default]
    Opened,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum FileStatus {
    Any,
    Opened,
    None,
    #[serde(rename = "Any!")]
    AnyBang,
    #[serde(rename = "Opened!")]
    OpenedBang,
    #[serde(rename = "None!")]
    NoneBang,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Event {
    OnChange,
    OnSave,
    None,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disable: Vec<Diagnostic>,
    //pub disable: Diagnostic
    #[serde(default = "Default::default", skip_serializing_if = "Vec::is_empty")]
    pub disable_scheme: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub globals: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group_file_status: BTreeMap<DiagnosticGroup, FileStatus>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group_severity: BTreeMap<DiagnosticGroup, GroupSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_files: Option<Files>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_files: Option<Files>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub needed_file_status: BTreeMap<Diagnostic, FileStatus>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub severity: BTreeMap<Diagnostic, Severity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_local_exclude: Vec<String>,
    #[serde(
        default = "Diagnostics::workspace_delay",
        skip_serializing_if = "Self::three_minute_validate"
    )]
    pub workspace_delay: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_event: Option<Event>,
    #[serde(
        default = "Diagnostics::workspace_rate",
        skip_serializing_if = "Self::full_percent_validate"
    )]
    pub workspace_rate: usize,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Diagnostics {
    #[inline]
    fn workspace_delay() -> usize {
        3000
    }

    #[inline]
    fn workspace_rate() -> usize {
        100
    }

    const fn three_minute_validate(ctx: &usize) -> bool {
        *ctx == 3000
    }

    const fn full_percent_validate(ctx: &usize) -> bool {
        *ctx == 100
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            enable: true,
            disable: Vec::default(),
            disable_scheme: Vec::default(),
            globals: Vec::default(),
            group_file_status: BTreeMap::default(),
            group_severity: BTreeMap::default(),
            ignored_files: None,
            library_files: None,
            unused_local_exclude: Vec::default(),
            workspace_delay: 3000,
            workspace_event: None,
            workspace_rate: 100,
            needed_file_status: BTreeMap::default(),
            severity: BTreeMap::default(),

            other: None,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub package_name: HashSet<String>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub private_name: HashSet<String>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub protected_name: HashSet<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub default_config: BTreeMap<Cow<'static, str>, Cow<'static, str>>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            enable: true,
            default_config: BTreeMap::default(),

            other: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Index {
    Enable,
    Auto,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ParamName {
    All,
    Literal,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum SemiColon {
    All,
    SameLine,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Hint {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_index: Option<Index>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub r#await: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_name: Option<ParamName>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub param_type: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semicolon: Option<SemiColon>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub set_type: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for Hint {
    fn default() -> Self {
        Self {
            enable: true,
            array_index: None,
            r#await: true,
            param_name: None,
            param_type: true,
            semicolon: None,
            set_type: false,

            other: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Hover {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default, skip_serializing_if = "Self::enum_limit_validate")]
    pub enums_limit: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub expand_alias: bool,
    #[serde(default, skip_serializing_if = "Self::preview_fields_validate")]
    pub preview_fields: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub view_number: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub view_string: bool,
    #[serde(default, skip_serializing_if = "Self::view_string_max_validate")]
    pub view_string_max: usize,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for Hover {
    fn default() -> Self {
        Self {
            enable: true,
            enums_limit: 5,
            expand_alias: true,
            preview_fields: 50,
            view_number: true,
            view_string: true,
            view_string_max: 1000,

            other: None,
        }
    }
}

impl Hover {
    const fn enum_limit_validate(ctx: &usize) -> bool {
        *ctx == 5
    }

    const fn preview_fields_validate(ctx: &usize) -> bool {
        *ctx == 50
    }

    const fn view_string_max_validate(ctx: &usize) -> bool {
        *ctx == 1000
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Misc {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Status {
    Default,
    Enable,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    Utf8,
    Ansi,
    Utf16le,
    Utf16be,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub builtin: BTreeMap<Cow<'static, str>, Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nonstandard_symbol: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub path_strict: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugin_args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub special: BTreeMap<Cow<'static, str>, Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub unicode_name: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Semantic {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub annotation: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub keyword: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub variable: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}
impl Default for Semantic {
    fn default() -> Self {
        Self {
            enable: true,
            annotation: true,
            keyword: false,
            variable: true,

            other: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelp {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}
impl Default for SignatureHelp {
    fn default() -> Self {
        Self {
            enable: true,
            other: None,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dict: Vec<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    #[serde(default, skip_serializing_if = "disabled")]
    pub cast_number_to_integer: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub weak_nil_check: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub weak_union_check: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub progress_bar: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub status_bar: bool,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            progress_bar: true,
            status_bar: true,
            other: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckThirdParty {
    Ask,
    Apply,
    ApplyInMemory,
    Disable,
    False,
}

impl Serialize for CheckThirdParty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Ask => serializer.serialize_str("Ask"),
            Self::Apply => serializer.serialize_str("Apply"),
            Self::ApplyInMemory => serializer.serialize_str("ApplyInMemory"),
            Self::Disable => serializer.serialize_str("Disable"),
            Self::False => serializer.serialize_bool(false),
        }
    }
}

impl<'de> Deserialize<'de> for CheckThirdParty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CheckThirdPartyVisitor;
        impl<'de> Visitor<'de> for CheckThirdPartyVisitor {
            type Value = CheckThirdParty;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, r#""Ask", "Apply", "ApplyInMemory", "Disable", or false"#)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v.as_str())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "Ask" => Ok(CheckThirdParty::Ask),
                    "Apply" => Ok(CheckThirdParty::Apply),
                    "ApplyInMemory" => Ok(CheckThirdParty::ApplyInMemory),
                    "Disable" => Ok(CheckThirdParty::Disable),
                    other => Err(serde::de::Error::custom(format!(
                        "unknown checkThirdParty value: {other}"
                    ))),
                }
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v {
                    return Err(serde::de::Error::custom(
                        "checkThirdParty cannot be set to `true`",
                    ));
                }
                Ok(CheckThirdParty::False)
            }
        }

        deserializer.deserialize_any(CheckThirdPartyVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_third_party: Option<CheckThirdParty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_dir: Vec<String>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub ignore_submodules: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<String>,
    #[serde(
        default = "Workspace::max_preload",
        skip_serializing_if = "Self::max_preload_validate"
    )]
    pub max_preload: usize,
    #[serde(
        default = "Workspace::preload_file_size",
        skip_serializing_if = "Self::preload_file_size_validate"
    )]
    pub preload_file_size: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub use_git_ignore: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_third_party: Vec<String>,

    /// This is added and custom to `llam`.
    ///
    /// resusing the `.luarc.json` file will reduce the number of files needed
    /// when developing a project.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub addons: BTreeMap<Cow<'static, str>, Addon>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl Workspace {
    pub const fn max_preload() -> usize {
        5000
    }

    pub const fn preload_file_size() -> usize {
        500
    }

    pub const fn max_preload_validate(ctx: &usize) -> bool {
        *ctx == 5000
    }

    pub const fn preload_file_size_validate(ctx: &usize) -> bool {
        *ctx == 500
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            check_third_party: None,
            ignore_dir: Vec::default(),
            ignore_submodules: true,
            library: Vec::default(),
            max_preload: 5000,
            preload_file_size: 500,
            use_git_ignore: true,
            user_third_party: Vec::default(),

            addons: BTreeMap::default(),

            other: None,
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LuaRc {
    #[serde(skip)]
    path: PathBuf,

    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_manager: Option<AddonManager>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<Completion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Diagnostics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<Doc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<Hint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<Hover>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misc: Option<Misc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<Runtime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic: Option<Semantic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help: Option<SignatureHelp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell: Option<Spell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
}

impl LuaRc {
    const LUARC: &'static str = ".luarc.json";

    pub fn detect(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();

        if dir.join(Self::LUARC).exists() {
            Self::read(&dir.join(Self::LUARC))
        } else {
            Self::new(dir)
        }
    }

    pub fn get_addons_mut(&mut self) -> &mut BTreeMap<Cow<'static, str>, Addon> {
        if self.workspace.is_none() {
            self.workspace = Some(Workspace {
                addons: BTreeMap::default(),
                ..Default::default()
            });
        }

        &mut self.workspace.as_mut().unwrap().addons
    }

    pub fn get_addons(&mut self) -> &BTreeMap<Cow<'static, str>, Addon> {
        if self.workspace.is_none() {
            self.workspace = Some(Workspace {
                addons: BTreeMap::default(),
                ..Default::default()
            });
        }

        &self.workspace.as_mut().unwrap().addons
    }

    pub fn update_addon(&mut self, addon: &Addon) {
        let name = addon.name();
        if let std::collections::btree_map::Entry::Vacant(e) =
            self.get_addons_mut().entry(name.clone())
        {
            e.insert(addon.clone());
        } else {
            self.get_addons_mut().get_mut(&name).unwrap().merge(addon);
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        Ok(std::fs::write(
            &self.path,
            serde_json::to_string_pretty(self)?,
        )?)
    }

    fn read(file: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(file)?;
        let mut lock: Self = serde_json::from_slice(&bytes)?;

        lock.path = file.to_path_buf();

        Ok(lock)
    }

    fn new(dir: &Path) -> Result<Self, Error> {
        // Attempt to read sha1 from cloned addon repositories
        let mut addons = BTreeMap::default();

        let _addons = dir.join(ADDONS_DIR);
        if _addons.exists() {
            for entry in (std::fs::read_dir(_addons)?).flatten() {
                if entry.path().join(".git").exists() && entry.path().join("config.json").exists() {
                    let output = std::process::Command::new("git")
                        .args(["rev-parse", "--verify", "HEAD"])
                        .output()?;

                    if output.status.success() {
                        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !sha.is_empty() {
                            let name = entry
                                .path()
                                .file_stem()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();
                            addons.insert(name.clone().into(), Addon::cats(name, Some(sha), None));
                            continue;
                        }
                    }

                    log::error!(
                        "checksum couldn't be retrieve for path: {}",
                        entry.path().display()
                    );
                    if !output.stderr.is_empty() {
                        log::error!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                } else if entry.path().is_dir() {
                    log::warn!("removing invalid addon: {}", entry.path().display());
                    std::fs::remove_dir_all(entry.path())?;
                } else if entry.path().is_file() {
                    log::warn!("removing invalid addon: {}", entry.path().display());
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        let lock = Self {
            path: dir.join(Self::LUARC),
            workspace: Some(Workspace {
                addons,
                ..Default::default()
            }),
            ..Default::default()
        };

        // TODO: Create error instead
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        log::debug!("creating lockfile {}", dir.join(Self::LUARC).display());
        std::fs::write(dir.join(Self::LUARC), serde_json::to_string_pretty(&lock)?)?;

        Ok(lock)
    }
}
