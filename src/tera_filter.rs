use std::collections::HashMap;

use artushak_web_assets::asset_cache::AssetCacheManifest;
use tera::{Error, Function, Result, Value};

pub fn get_tera_function_get_asset_url(
    asset_cache_manifest: AssetCacheManifest,
    base_url: String,
) -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> Result<Value> {
        match args.get("name") {
            Some(asset_name) => {
                let base_url = base_url.clone();

                let asset_name_string = asset_name
                    .as_str()
                    .ok_or_else(|| Error::msg("Parameter 'name' should be string"))?;

                let asset_path = asset_cache_manifest
                    .get_entry(asset_name_string)
                    .map(|cache_entry| cache_entry.path);
                if let Some(asset_path_real) = asset_path {
                    Ok(Value::String(base_url + asset_path_real.to_str().unwrap()))
                } else {
                    Err(format!("Asset {} not found", &asset_name_string).into())
                }
            }
            None => Err("Parameter 'name' is required".into()),
        }
    })
}
