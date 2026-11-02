//! Schema 2 desktop pipeline choices, independent of legacy role configuration.
use crate::{
    ErrorCode,
    config::{
        ComputeDevice, DisplayMode, MAX_CONFIG_BYTES, RuntimeRoot, TargetLanguage,
        valid_identifier, valid_path,
    },
};
use serde::Deserialize;
use std::{
    fmt,
    io::Read,
    path::{Path, PathBuf},
};

pub const SCHEMA_VERSION: u16 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Language {
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeviceSelection {
    Default {},
    Device { device_id: String },
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioSource {
    Microphone { selection: DeviceSelection },
    System { selection: DeviceSelection },
}

#[derive(Clone, Copy, Deserialize)]
pub enum FunAsrRegion {
    #[serde(rename = "cn-beijing")]
    Beijing,
    #[serde(rename = "ap-southeast-1")]
    Singapore,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case", deny_unknown_fields)]
pub enum Recognition {
    Local {
        model: PathBuf,
        device: ComputeDevice,
    },
    FunAsr {
        region: FunAsrRegion,
        model: String,
        api_key_env: String,
        #[serde(default)]
        workspace_id: Option<String>,
        #[serde(default = "default_silence")]
        sentence_silence_ms: u32,
        #[serde(default)]
        vocabulary_id: Option<String>,
        #[serde(default)]
        context: Option<String>,
    },
}

fn default_silence() -> u32 {
    800
}

#[derive(Clone, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case", deny_unknown_fields)]
pub enum Translation {
    Disabled {},
    Local {
        model: PathBuf,
        device: ComputeDevice,
        target_language: TargetLanguage,
    },
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    #[serde(default = "source_only")]
    pub mode: DisplayMode,
    #[serde(default = "default_lines")]
    pub visible_lines: usize,
}
fn source_only() -> DisplayMode {
    DisplayMode::SourceOnly
}
fn default_lines() -> usize {
    3
}
impl Default for Presentation {
    fn default() -> Self {
        Self {
            mode: source_only(),
            visible_lines: default_lines(),
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    pub schema_version: u16,
    pub source_language: Language,
    pub capture: AudioSource,
    pub asr: Recognition,
    pub translation: Translation,
    #[serde(default)]
    pub overlay: Presentation,
}

impl fmt::Debug for PipelineConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PipelineConfig { configuration: [redacted] }")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineError {
    pub code: ErrorCode,
    pub field: &'static str,
}
impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {} (pipeline schema 2)", self.code, self.field)
    }
}
impl std::error::Error for PipelineError {}
fn invalid(field: &'static str) -> PipelineError {
    PipelineError {
        code: ErrorCode::InvalidConfig,
        field,
    }
}

impl PipelineConfig {
    /// Structural validation only: no environment-value or runtime access.
    pub fn parse(bytes: &[u8]) -> Result<Self, PipelineError> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(invalid("document"));
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| invalid("document"))?;
        value.validate()?;
        Ok(value)
    }

    /// Relative model paths use the canonical configuration file's parent.
    pub fn load(path: &Path) -> Result<Self, PipelineError> {
        let absolute = path.canonicalize().map_err(|_| invalid("file"))?;
        let mut bytes = Vec::new();
        std::fs::File::open(&absolute)
            .map_err(|_| invalid("file"))?
            .take((MAX_CONFIG_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| invalid("file"))?;
        let mut value = Self::parse(&bytes)?;
        value.resolve_paths(absolute.parent().ok_or_else(|| invalid("file"))?)?;
        Ok(value)
    }

    pub fn resolve_paths(&mut self, directory: &Path) -> Result<(), PipelineError> {
        let root = RuntimeRoot::new(directory.to_path_buf()).map_err(|_| invalid("directory"))?;
        if let Recognition::Local { model, .. } = &mut self.asr {
            *model = root.resolve(model).map_err(|_| invalid("asr"))?;
        }
        if let Translation::Local { model, .. } = &mut self.translation {
            *model = root.resolve(model).map_err(|_| invalid("translation"))?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), PipelineError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(PipelineError {
                code: ErrorCode::UnsupportedVersion,
                field: "schema_version",
            });
        }
        let selection = match &self.capture {
            AudioSource::Microphone { selection } | AudioSource::System { selection } => selection,
        };
        if let DeviceSelection::Device { device_id } = selection {
            identifier(device_id, "capture")?;
        }
        match &self.asr {
            Recognition::Local { model, device } => local(model, device, "asr")?,
            Recognition::FunAsr {
                model,
                api_key_env,
                workspace_id,
                sentence_silence_ms,
                vocabulary_id,
                context,
                ..
            } => {
                identifier(model, "asr")?;
                if !environment_name(api_key_env) || !(200..=6000).contains(sentence_silence_ms) {
                    return Err(invalid("asr"));
                }
                for value in [workspace_id, vocabulary_id].into_iter().flatten() {
                    identifier(value, "asr")?;
                }
                if let Some(value) = context
                    && (value.chars().count() > 400 || value.chars().any(char::is_control))
                {
                    return Err(invalid("asr"));
                }
            }
        }
        if let Translation::Local { model, device, .. } = &self.translation {
            local(model, device, "translation")?;
        } else if self.overlay.mode != DisplayMode::SourceOnly {
            return Err(invalid("overlay"));
        }
        if !(1..=6).contains(&self.overlay.visible_lines) {
            return Err(invalid("overlay"));
        }
        Ok(())
    }
}

fn environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 128
        && bytes
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
fn identifier(value: &str, field: &'static str) -> Result<(), PipelineError> {
    if !valid_identifier(value) {
        return Err(invalid(field));
    }
    Ok(())
}
fn local(model: &Path, device: &ComputeDevice, field: &'static str) -> Result<(), PipelineError> {
    if !valid_path(model) {
        return Err(invalid(field));
    }
    if let ComputeDevice::Vulkan {
        vendor_id,
        device_id,
        device_name,
    } = device
    {
        if *vendor_id == 0 || *device_id == 0 {
            return Err(invalid(field));
        }
        identifier(device_name, field)?;
    }
    Ok(())
}
