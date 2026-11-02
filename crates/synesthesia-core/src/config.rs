//! Strict v1 role configuration. Neither parse errors nor Debug expose input.
use crate::{ErrorCode, MAX_HISTORY};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    io::Read,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};

pub const CONFIG_VERSION: u16 = 1;
pub const MAX_CONFIG_BYTES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Full,
    Server,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetLanguage {
    #[serde(rename = "zh-Hans")]
    SimplifiedChinese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    #[default]
    Bilingual,
    SourceOnly,
    TranslationOnly,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayConfig {
    #[serde(default)]
    pub mode: DisplayMode,
    #[serde(default = "default_history")]
    pub history: usize,
}
fn default_history() -> usize {
    3
}
impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Bilingual,
            history: 3,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum CaptureConfig {
    FollowDefault {},
    Device { device_id: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case", deny_unknown_fields)]
pub enum ComputeDevice {
    Cpu {},
    Vulkan {
        vendor_id: u32,
        device_id: u32,
        device_name: String,
    },
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceConfig {
    pub asr_model: PathBuf,
    pub translation_model: PathBuf,
    pub device: ComputeDevice,
}
impl fmt::Debug for InferenceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InferenceConfig { paths: [redacted], device: [configured] }")
    }
}

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);
impl Secret {
    /// Only the authentication boundary should call this; never format it.
    pub fn expose_for_authentication(&self) -> &str {
        &self.0
    }
    fn valid(&self) -> bool {
        (16..=256).contains(&self.0.len()) && self.0.bytes().all(|b| b.is_ascii_graphic())
    }
}
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionConfig {
    pub address: String,
    pub token: Secret,
}
impl fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectionConfig { address: [redacted], token: [redacted] }")
    }
}

#[derive(Debug, Clone)]
pub enum Config {
    Full {
        schema_version: u16,
        source: SourceLanguage,
        target: TargetLanguage,
        inference: InferenceConfig,
        capture: CaptureConfig,
        overlay: OverlayConfig,
    },
    Server {
        schema_version: u16,
        source: SourceLanguage,
        target: TargetLanguage,
        inference: InferenceConfig,
        listen: SocketAddr,
        token: Secret,
    },
    Client {
        schema_version: u16,
        source: SourceLanguage,
        target: TargetLanguage,
        connection: ConnectionConfig,
        capture: CaptureConfig,
        overlay: OverlayConfig,
    },
}

// Parse directly as a strict struct to retain the original field/line location.
// Internally tagged enum buffering would lose nested paths. Explicit null is
// rejected even for optional role fields so a forbidden field cannot hide in it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    role: Role,
    schema_version: u16,
    source: SourceLanguage,
    target: TargetLanguage,
    #[serde(default, deserialize_with = "present")]
    inference: Option<InferenceConfig>,
    #[serde(default, deserialize_with = "present")]
    capture: Option<CaptureConfig>,
    #[serde(default, deserialize_with = "present")]
    overlay: Option<OverlayConfig>,
    #[serde(default, deserialize_with = "present")]
    listen: Option<SocketAddr>,
    #[serde(default, deserialize_with = "present")]
    token: Option<Secret>,
    #[serde(default, deserialize_with = "present")]
    connection: Option<ConnectionConfig>,
}
fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}
fn required<T>(value: Option<T>, field: &'static str) -> Result<T, ConfigError> {
    value.ok_or_else(|| error(ErrorCode::InvalidConfig, field))
}
impl RawConfig {
    fn into_config(self) -> Result<Config, ConfigError> {
        let forbidden = match self.role {
            Role::Full => [
                (self.listen.is_some(), "listen"),
                (self.token.is_some(), "token"),
                (self.connection.is_some(), "connection"),
            ]
            .into_iter()
            .find(|(present, _)| *present),
            Role::Server => [
                (self.capture.is_some(), "capture"),
                (self.overlay.is_some(), "overlay"),
                (self.connection.is_some(), "connection"),
            ]
            .into_iter()
            .find(|(present, _)| *present),
            Role::Client => [
                (self.inference.is_some(), "inference"),
                (self.listen.is_some(), "listen"),
                (self.token.is_some(), "token"),
            ]
            .into_iter()
            .find(|(present, _)| *present),
        };
        if let Some((_, field)) = forbidden {
            return Err(error(ErrorCode::InvalidConfig, field));
        }
        Ok(match self.role {
            Role::Full => Config::Full {
                schema_version: self.schema_version,
                source: self.source,
                target: self.target,
                inference: required(self.inference, "inference")?,
                capture: required(self.capture, "capture")?,
                overlay: self.overlay.unwrap_or_default(),
            },
            Role::Server => Config::Server {
                schema_version: self.schema_version,
                source: self.source,
                target: self.target,
                inference: required(self.inference, "inference")?,
                listen: required(self.listen, "listen")?,
                token: required(self.token, "token")?,
            },
            Role::Client => Config::Client {
                schema_version: self.schema_version,
                source: self.source,
                target: self.target,
                connection: required(self.connection, "connection")?,
                capture: required(self.capture, "capture")?,
                overlay: self.overlay.unwrap_or_default(),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub code: ErrorCode,
    pub field: &'static str,
    pub line: usize,
    pub column: usize,
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {} (line {}, column {}); v1 requires source en/ja and target zh-Hans",
            self.code, self.field, self.line, self.column
        )
    }
}
impl std::error::Error for ConfigError {}
fn error(code: ErrorCode, field: &'static str) -> ConfigError {
    ConfigError {
        code,
        field,
        line: 0,
        column: 0,
    }
}

fn safe_field(path: &serde_path_to_error::Path) -> &'static str {
    // Unknown field names may contain secrets, so whitelist the path too.
    let value = path.to_string();
    for field in [
        "schema_version",
        "source",
        "target",
        "inference",
        "connection",
        "capture",
        "overlay",
        "listen",
        "token",
        "role",
    ] {
        if value.split('.').any(|part| part == field) {
            return field;
        }
    }
    "document"
}

impl Config {
    pub fn parse(bytes: &[u8], expected: Role) -> Result<Self, ConfigError> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(error(ErrorCode::InvalidConfig, "document"));
        }
        let mut de = serde_json::Deserializer::from_slice(bytes);
        let raw: RawConfig =
            serde_path_to_error::deserialize(&mut de).map_err(|e| ConfigError {
                code: ErrorCode::InvalidConfig,
                field: safe_field(e.path()),
                line: e.inner().line(),
                column: e.inner().column(),
            })?;
        de.end().map_err(|e| ConfigError {
            code: ErrorCode::InvalidConfig,
            field: "document",
            line: e.line(),
            column: e.column(),
        })?;
        let config = raw.into_config()?;
        config.validate(expected)?;
        Ok(config)
    }
    pub fn role(&self) -> Role {
        match self {
            Self::Full { .. } => Role::Full,
            Self::Server { .. } => Role::Server,
            Self::Client { .. } => Role::Client,
        }
    }
    pub fn source(&self) -> SourceLanguage {
        match self {
            Self::Full { source, .. }
            | Self::Server { source, .. }
            | Self::Client { source, .. } => *source,
        }
    }
    pub fn inference(&self) -> Option<&InferenceConfig> {
        match self {
            Self::Full { inference, .. } | Self::Server { inference, .. } => Some(inference),
            Self::Client { .. } => None,
        }
    }
    fn validate(&self, expected: Role) -> Result<(), ConfigError> {
        let version = match self {
            Self::Full { schema_version, .. }
            | Self::Server { schema_version, .. }
            | Self::Client { schema_version, .. } => *schema_version,
        };
        if version != CONFIG_VERSION {
            return Err(error(ErrorCode::UnsupportedVersion, "schema_version"));
        }
        if self.role() != expected {
            return Err(error(ErrorCode::WrongRole, "role"));
        }
        if let Some(inference) = self.inference() {
            for model in [&inference.asr_model, &inference.translation_model] {
                if !valid_path(model) {
                    return Err(error(ErrorCode::InvalidPath, "inference"));
                }
            }
            if let ComputeDevice::Vulkan {
                vendor_id,
                device_id,
                device_name,
            } = &inference.device
                && (*vendor_id == 0 || *device_id == 0 || !valid_identifier(device_name))
            {
                return Err(error(ErrorCode::InvalidConfig, "inference"));
            }
        }
        match self {
            Self::Full {
                capture, overlay, ..
            } => validate_desktop(capture, overlay)?,
            Self::Server { listen, token, .. } => {
                if listen.port() == 0 {
                    return Err(error(ErrorCode::InvalidAddress, "listen"));
                }
                if !token.valid() {
                    return Err(error(ErrorCode::InvalidToken, "token"));
                }
            }
            Self::Client {
                capture,
                overlay,
                connection,
                ..
            } => {
                validate_desktop(capture, overlay)?;
                let url = url::Url::parse(&connection.address)
                    .map_err(|_| error(ErrorCode::InvalidAddress, "connection"))?;
                if url.scheme() != "ws"
                    || url.host_str().is_none()
                    // Url normalizes empty userinfo away (`ws://@host`). Check
                    // the original authority so every userinfo form is banned.
                    || connection.address.split_once("://").is_some_and(|(_, rest)| {
                        rest.split(['/', '?', '#']).next().is_some_and(|authority| authority.contains('@'))
                    })
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                    || url.port() == Some(0)
                    || connection.address.len() > 2048
                {
                    return Err(error(ErrorCode::InvalidAddress, "connection"));
                }
                if !connection.token.valid() {
                    return Err(error(ErrorCode::InvalidToken, "connection"));
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
fn validate_desktop(capture: &CaptureConfig, overlay: &OverlayConfig) -> Result<(), ConfigError> {
    if !(1..=MAX_HISTORY).contains(&overlay.history) {
        return Err(error(ErrorCode::InvalidConfig, "overlay"));
    }
    if let CaptureConfig::Device { device_id } = capture
        && !valid_identifier(device_id)
    {
        return Err(error(ErrorCode::InvalidConfig, "capture"));
    }
    Ok(())
}
pub(crate) fn valid_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.as_os_str().to_string_lossy().contains('\0')
        && (path.is_absolute()
            || path
                .components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)))
}

/// Always absolute; callers derive it from the executable, never process CWD.
#[derive(Clone)]
pub struct RuntimeRoot(PathBuf);
impl RuntimeRoot {
    pub fn new(root: PathBuf) -> Result<Self, ConfigError> {
        if !root.is_absolute() {
            return Err(error(ErrorCode::InvalidPath, "runtime_root"));
        }
        Ok(Self(root))
    }
    pub fn from_executable(executable: &Path) -> Result<Self, ConfigError> {
        Self::new(
            executable
                .parent()
                .ok_or_else(|| error(ErrorCode::InvalidPath, "runtime_root"))?
                .to_path_buf(),
        )
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn resolve(&self, path: &Path) -> Result<PathBuf, ConfigError> {
        if !valid_path(path) {
            return Err(error(ErrorCode::InvalidPath, "path"));
        }
        Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.0.join(path)
        })
    }
    pub fn load(&self, path: &Path, role: Role) -> Result<Config, ConfigError> {
        let file = std::fs::File::open(self.resolve(path)?)
            .map_err(|_| error(ErrorCode::InvalidConfig, "file"))?;
        let mut bytes = Vec::new();
        file.take((MAX_CONFIG_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| error(ErrorCode::InvalidConfig, "file"))?;
        let mut config = Config::parse(&bytes, role)?;
        match &mut config {
            Config::Full { inference, .. } | Config::Server { inference, .. } => {
                inference.asr_model = self.resolve(&inference.asr_model)?;
                inference.translation_model = self.resolve(&inference.translation_model)?;
            }
            Config::Client { .. } => {}
        }
        Ok(config)
    }
}
