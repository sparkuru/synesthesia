use serde_json::{Value, json};
use std::path::Path;
use synesthesia_core::{
    ErrorCode,
    config::MAX_CONFIG_BYTES,
    pipeline::{PipelineConfig, Recognition, Translation},
};

fn cloud() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../config/examples/microphone-fun-asr.json"
    ))
    .unwrap()
}
fn parse(value: &Value) -> Result<PipelineConfig, synesthesia_core::pipeline::PipelineError> {
    PipelineConfig::parse(&serde_json::to_vec(value).unwrap())
}
#[test]
fn input_recognition_and_translation_are_independent_choices() {
    for input in ["microphone", "system"] {
        for local_asr in [false, true] {
            for translate in [false, true] {
                let mut config = cloud();
                config["capture"]["kind"] = json!(input);
                if local_asr {
                    config["asr"] = json!({"provider":"local","model":"models/asr.gguf","device":{"backend":"cpu"}});
                }
                if translate {
                    config["translation"] = json!({"provider":"local","model":"models/translation.gguf","device":{"backend":"cpu"},"target_language":"zh-Hans"});
                    config["overlay"]["mode"] = json!("bilingual");
                }
                let parsed = parse(&config).unwrap();
                assert_eq!(matches!(parsed.asr, Recognition::Local { .. }), local_asr);
                assert_eq!(
                    matches!(parsed.translation, Translation::Local { .. }),
                    translate
                );
            }
        }
    }
}
#[test]
fn examples_are_accepted_without_model_or_credential_access() {
    for bytes in [
        include_bytes!("../../../config/examples/microphone-fun-asr.json").as_slice(),
        include_bytes!("../../../config/examples/system-fun-asr.json").as_slice(),
        include_bytes!("../../../config/examples/microphone-local.json").as_slice(),
        include_bytes!("../../../config/examples/system-local-translation.json").as_slice(),
    ] {
        PipelineConfig::parse(bytes).unwrap();
    }
    let mut value = cloud();
    value["asr"]["api_key_env"] = json!("SYNESTHESIA_NONEXISTENT_CONFIG_CHECK_KEY");
    assert!(parse(&value).is_ok());
}
#[test]
fn disabled_translation_rejects_hidden_model_requirements_and_unavailable_views() {
    for mode in ["bilingual", "translation_only"] {
        let mut value = cloud();
        value["overlay"]["mode"] = json!(mode);
        assert_eq!(parse(&value).unwrap_err().field, "overlay");
    }
    let mut value = cloud();
    value["translation"]["model"] = json!("unwanted.gguf");
    assert!(parse(&value).is_err());
    value = cloud();
    value["asr"]["model_path"] = json!("unwanted.gguf");
    assert!(parse(&value).is_err());
}
#[test]
fn secrets_and_malformed_fields_are_not_echoed() {
    let marker = "do-not-log-this-private-value";
    let mut value = cloud();
    value["asr"]["api_key"] = json!(marker);
    let error = parse(&value).unwrap_err();
    assert!(!format!("{error:?} {error}").contains(marker));
    value = cloud();
    value["asr"]["context"] = json!(marker);
    assert!(!format!("{:?}", parse(&value).unwrap()).contains(marker));
    value["asr"]["api_key_env"] = json!(marker);
    assert_eq!(parse(&value).unwrap_err().field, "asr");
}
#[test]
fn strict_schema_rejects_duplicates_unknowns_old_versions_and_oversize() {
    let bytes = serde_json::to_string(&cloud()).unwrap();
    let duplicate = bytes.replacen(
        "\"schema_version\":2",
        "\"schema_version\":2,\"schema_version\":2",
        1,
    );
    assert_ne!(bytes, duplicate);
    assert!(PipelineConfig::parse(duplicate.as_bytes()).is_err());
    assert!(PipelineConfig::parse(format!("{bytes} trailing").as_bytes()).is_err());
    assert!(PipelineConfig::parse(&vec![b' '; MAX_CONFIG_BYTES + 1]).is_err());
    let mut value = cloud();
    value["schema_version"] = json!(1);
    assert_eq!(
        parse(&value).unwrap_err().code,
        ErrorCode::UnsupportedVersion
    );
    value = cloud();
    value["capture"]["selection"]["unexpected"] = json!(true);
    assert!(parse(&value).is_err());
    value = cloud();
    value["capture"]["selection"] = json!({"mode":"device","device_id":""});
    assert_eq!(parse(&value).unwrap_err().field, "capture");
}
#[test]
fn provider_parameters_and_display_bounds_are_validated() {
    for silence in [199, 6001] {
        let mut value = cloud();
        value["asr"]["sentence_silence_ms"] = json!(silence);
        assert!(parse(&value).is_err());
    }
    for lines in [0, 7] {
        let mut value = cloud();
        value["overlay"]["visible_lines"] = json!(lines);
        assert!(parse(&value).is_err());
    }
    let mut value = cloud();
    value["asr"]["context"] = json!("字".repeat(400));
    assert!(parse(&value).is_ok());
    value["asr"]["context"] = json!("字".repeat(401));
    assert!(parse(&value).is_err());
}
#[test]
fn local_paths_are_config_relative_without_loading_models() {
    let bytes = include_bytes!("../../../config/examples/system-local-translation.json");
    let mut config = PipelineConfig::parse(bytes).unwrap();
    let root = if cfg!(windows) {
        Path::new("C:\\uncreated-runtime")
    } else {
        Path::new("/uncreated-runtime")
    };
    config.resolve_paths(root).unwrap();
    match config.asr {
        Recognition::Local { model, .. } => assert_eq!(model, root.join("models/asr.gguf")),
        _ => panic!("expected local ASR"),
    }
    match config.translation {
        Translation::Local { model, .. } => assert_eq!(model, root.join("models/translation.gguf")),
        _ => panic!("expected local translation"),
    }
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    value["asr"]["model"] = json!("../outside.gguf");
    assert!(parse(&value).is_err());
    value["asr"]["model"] = json!("models/asr.gguf");
    value["asr"]["device"] =
        json!({"backend":"vulkan","vendor_id":0,"device_id":1,"device_name":"GPU"});
    assert!(parse(&value).is_err());
}
