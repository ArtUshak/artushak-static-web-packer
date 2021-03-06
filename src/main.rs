pub mod filters;
pub mod manifest;
pub mod tera_filter;
mod test;

use std::{
    collections::HashMap,
    fs::{copy, File},
    io::{self, Write},
    path::{PathBuf, Path},
    str::FromStr,
};

use artushak_web_assets::{
    asset_config::AssetConfig,
    asset_filter::{AssetFilter, AssetFilterRegistry},
    assets::{AssetError, AssetFilterError},
    load_cache_manifest, pack,
};
use clap::Parser;
use filters::{
    run_executable::AssetFilterRunExecutable, scss2css::AssetFilterSCSS, AssetFilterCustomError,
};
use log::info;
use manifest::StaticWebsiteManifest;
use serde_json::from_reader;
use tera::{Context, Tera};
use tera_filter::get_tera_function_get_asset_url;

#[derive(Parser, Debug)]
#[clap(author = "Artiom Khandamirov <t9max@yandex.ru>")]
struct CLIOptions {
    #[clap(short, long)]
    manifest_file: Option<PathBuf>,
}

pub fn process_manifest<E>(
    manifest: StaticWebsiteManifest,
    initial_context: Context,
    asset_filters: AssetFilterRegistry<E>,
) -> Result<(), StatickPackError<E>>
where
    E: AssetFilterError,
{
    env_logger::init();

    let asset_config = AssetConfig {
        target_directory_path: manifest.static_directory_path,
        internal_directory_path: manifest.internal_directory_path,
        source_directory_path: manifest.asset_directory_path,
    };

    info!("Processing assets...");
    pack(
        &manifest.asset_manifest_path,
        &manifest.asset_cache_manifest_path,
        &asset_config,
        &asset_filters,
    )?;

    let asset_manifest = load_cache_manifest(&manifest.asset_cache_manifest_path)?;

    info!("Initializing Tera...");
    let mut tera = Tera::new(&manifest.tera_input_directory)?;

    tera.register_function(
        "get_asset_url",
        get_tera_function_get_asset_url(asset_manifest, manifest.static_base_url),
    );

    let mut context = initial_context;
    for (key, value) in manifest.context.iter() {
        context.insert(key, value);
    }

    info!("Processing Tera templates...");
    for template_entry in manifest.tera_templates {
        let mut local_context = context.clone();
        for (key, value) in template_entry.context.iter() {
            local_context.insert(key, value);
        }

        let rendered_html = tera.render(&template_entry.template_name, &context)?;

        let mut output_file = File::create(
            manifest
                .html_output_directory_path
                .join(template_entry.output_name),
        )?;
        output_file.write_all(rendered_html.as_bytes())?;
    }

    info!("Copying other files...");
    for copy_path in manifest.copy_paths {
        copy(
            manifest.copy_input_directory_path.join(&copy_path),
            manifest.copy_output_directory_path.join(&copy_path),
        )?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum StatickPackError<E>
where
    E: AssetFilterError,
{
    IOError(io::Error),
    TeraError(tera::Error),
    AssetError(AssetError<E>),
    JSONError(serde_json::Error),
}

impl<E> From<tera::Error> for StatickPackError<E>
where
    E: AssetFilterError,
{
    fn from(err: tera::Error) -> Self {
        StatickPackError::TeraError(err)
    }
}

impl<E> From<AssetError<E>> for StatickPackError<E>
where
    E: AssetFilterError,
{
    fn from(err: AssetError<E>) -> Self {
        StatickPackError::AssetError(err)
    }
}

impl<E> From<io::Error> for StatickPackError<E>
where
    E: AssetFilterError,
{
    fn from(err: io::Error) -> Self {
        StatickPackError::IOError(err)
    }
}

impl<E> From<serde_json::Error> for StatickPackError<E>
where
    E: AssetFilterError,
{
    fn from(err: serde_json::Error) -> Self {
        StatickPackError::JSONError(err)
    }
}

fn run(manifest_file_path: &Path) -> Result<(), StatickPackError<AssetFilterCustomError>> {
    let manifest: StaticWebsiteManifest;
    {
        let manifest_file = File::open(manifest_file_path)?;
        manifest = from_reader(manifest_file)?;
    }

    // TODO
    let mut asset_filters_map: HashMap<String, Box<dyn AssetFilter<AssetFilterCustomError>>> =
        HashMap::new();
    asset_filters_map.insert(
        "SCSS2CSS".to_string(),
        Box::new(AssetFilterSCSS {
            format: Default::default(),
        }),
    );
    asset_filters_map.insert(
        "RunExecutable".to_string(),
        Box::new(AssetFilterRunExecutable {}),
    );

    let asset_filter_registry = AssetFilterRegistry::new(asset_filters_map);

    process_manifest(manifest, Context::new(), asset_filter_registry)?;

    Ok(())
}

fn main() {
    let opts = CLIOptions::parse();

    let manifest_file_path = opts
        .manifest_file
        .unwrap_or_else(|| PathBuf::from_str("static_website.json").unwrap());

        run(&manifest_file_path).unwrap();
}
