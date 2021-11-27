pub mod filters;
pub mod manifest;
pub mod tera_filter;

use std::{
    collections::HashMap,
    fs::{copy, File},
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

use artushak_web_assets::{
    asset_config::AssetConfig,
    asset_filter::{AssetFilter, AssetFilterRegistry},
    assets::{AssetError, AssetFilterError},
    load_cache_manifest, pack,
};
use clap::Clap;
use filters::{scss2css::AssetFilterSCSS, AssetFilterCustomError};
use manifest::StaticWebsiteManifest;
use serde_json::from_reader;
use tera::{Context, Tera};
use tera_filter::get_tera_function_get_asset_url;

#[derive(Clap)]
#[clap(author = "Artiom Khandamirov <t9max@yandex.ru>")]
#[clap(setting = clap::AppSettings::ColoredHelp)]
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
    let asset_config = AssetConfig {
        target_directory_path: manifest.static_directory_path,
        internal_directory_path: manifest.internal_directory_path,
        source_directory_path: manifest.asset_directory_path,
    };

    println!("Processing assets...");
    pack(
        &manifest.asset_manifest_path,
        &manifest.asset_cache_manifest_path,
        &asset_config,
        &asset_filters,
    )?;

    let asset_manifest = load_cache_manifest(&manifest.asset_cache_manifest_path)?;

    println!("Initializing Tera...");
    let mut tera = Tera::new(&manifest.tera_input_directory)?;

    tera.register_function(
        "get_asset_url",
        get_tera_function_get_asset_url(asset_manifest, manifest.static_base_url),
    );

    let mut context = initial_context;
    for (key, value) in manifest.context.iter() {
        context.insert(key, value);
    }

    println!("Processing Tera templates...");
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

    println!("Copying other files...");
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

fn main() {
    let opts = CLIOptions::parse();

    let manifest_file_path = opts
        .manifest_file
        .unwrap_or_else(|| PathBuf::from_str("static_website.json").unwrap());

    let manifest: StaticWebsiteManifest;
    {
        let manifest_file = File::open(manifest_file_path).unwrap();
        manifest = from_reader(manifest_file).unwrap();
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

    let asset_filter_registry = AssetFilterRegistry::new(asset_filters_map);

    process_manifest(manifest, Context::new(), asset_filter_registry).unwrap();
}
