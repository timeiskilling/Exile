use sherpa_onnx::OnlineRecognizerConfig;
use std::path::{Path, PathBuf};

fn model_file(model_dir: &Path, filename: &str) -> String {
    let path = model_dir.join(filename);

    assert!(path.is_file(), "No Found model file: {}", path.display());

    path.to_string_lossy().into_owned()
}

pub fn create_instance_config() -> OnlineRecognizerConfig {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = crate_dir.parent().expect("exile must be inside rustExile");
    let hotwords_path = workspace_dir.join("hotwords.txt");

    assert!(
        hotwords_path.is_file(),
        "Not found hotwords-файл: {}",
        hotwords_path.display()
    );

    let model_dir = workspace_dir
        .join("sherpa-test")
        .join("sherpa-onnx-streaming-zipformer-en-2023-06-26");

    assert!(
        model_dir.is_dir(),
        "Directory not found: {}",
        model_dir.display()
    );

    let mut config = OnlineRecognizerConfig::default();

    config.model_config.transducer.encoder = Some(model_file(
        &model_dir,
        "encoder-epoch-99-avg-1-chunk-16-left-128.int8.onnx",
    ));
    config.model_config.transducer.decoder = Some(model_file(
        &model_dir,
        "decoder-epoch-99-avg-1-chunk-16-left-128.onnx",
    ));
    config.model_config.transducer.joiner = Some(model_file(
        &model_dir,
        "joiner-epoch-99-avg-1-chunk-16-left-128.int8.onnx",
    ));
    config.model_config.tokens = Some(model_file(&model_dir, "tokens.txt"));
    config.model_config.provider = Some("cpu".to_owned());
    config.model_config.num_threads = 2;
    config.model_config.debug = true;
    config.enable_endpoint = true;
    config.decoding_method = Some("modified_beam_search".to_owned());
    config.hotwords_file = Some(hotwords_path.to_string_lossy().into_owned());
    config.max_active_paths = 4;
    config
}
