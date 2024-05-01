use candle_core::{
    utils::{cuda_is_available, metal_is_available},
    Device,
};

/// Loads the safetensors files for a model from the hub based on a json index file.
pub async fn hub_load_safetensors(
    repo: &hf_hub::api::tokio::ApiRepo,
    json_file: &str,
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let json_file = repo.get(json_file).await?;
    let json_file = std::fs::File::open(json_file)?;
    let json: serde_json::Value = serde_json::from_reader(&json_file)?;
    let weight_map = match json.get("weight_map") {
        None => anyhow::bail!("no weight map in {json_file:?}"),
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => anyhow::bail!("weight map in {json_file:?} is not a map"),
    };
    let mut safetensors_files = std::collections::HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            safetensors_files.insert(file.to_string());
        }
    }

    let safetensors_files =
        futures::future::try_join_all(safetensors_files.iter().map(|v| repo.get(v))).await?;

    Ok(safetensors_files)
}

/// Picks the device to use for generation depending on what's available.
pub fn choose_device() -> anyhow::Result<Device> {
    if metal_is_available() {
        Ok(Device::new_metal(0)?)
    } else if cuda_is_available() {
        Ok(Device::new_cuda(0)?)
    } else {
        return Ok(Device::Cpu);
    }
}
