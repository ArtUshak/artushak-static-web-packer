#[cfg(test)]
mod tests {
    use crate::{manifest::StaticWebsiteManifest, run};
    use std::{
        fs::{create_dir, read_to_string, File},
        path::Path,
    };

    use html5ever::tree_builder::QuirksMode;
    use scraper::{ElementRef, Html, Selector};
    use tempfile::TempDir;

    #[test]
    fn test1() {
        let test_directory_path = Path::new("test_files");

        let temp_directory = TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path();

        let correct_output_directory_path = test_directory_path.join("correct_output");
        let output_directory_path = temp_directory_path.join("output");
        create_dir(&output_directory_path).unwrap();
        let static_directory_path = output_directory_path.join("static");
        let internal_directory_path = temp_directory_path.join("internal");
        create_dir(&internal_directory_path).unwrap();
        let manifest_file_path = temp_directory_path.join("static_website.json");
        let asset_cache_manifest_path = internal_directory_path.join("assets_cache.json");

        let base_manifest_file_path = test_directory_path.join("static_website.json");
        let mut base_manifest: StaticWebsiteManifest;
        {
            let base_manifest_file = File::open(base_manifest_file_path).unwrap();
            base_manifest = serde_json::from_reader(base_manifest_file).unwrap();
        }
        base_manifest.internal_directory_path = internal_directory_path.clone();
        base_manifest.copy_output_directory_path = output_directory_path.clone();
        base_manifest.html_output_directory_path = output_directory_path.clone();
        base_manifest.static_directory_path = static_directory_path.clone();
        base_manifest.asset_cache_manifest_path = asset_cache_manifest_path;

        {
            let manifest_file = File::create(&manifest_file_path).unwrap();
            serde_json::to_writer(manifest_file, &base_manifest).unwrap();
        }

        run(&manifest_file_path).unwrap();

        let correct_output_html_file_path = correct_output_directory_path.join("index.html");
        let correct_output_html_string = read_to_string(correct_output_html_file_path).unwrap();

        let correct_output_html = Html::parse_document(&correct_output_html_string);

        assert_eq!(correct_output_html.quirks_mode, QuirksMode::NoQuirks);
        let correct_output_html_body_elements: Vec<ElementRef> = correct_output_html
            .select(&Selector::parse("body").unwrap())
            .collect();
        assert_eq!(correct_output_html_body_elements.len(), 1);
        let correct_output_html_body = correct_output_html_body_elements.get(0).unwrap();

        let output_html_file_path = output_directory_path.join("index.html");
        let output_html_string = read_to_string(output_html_file_path).unwrap();

        let output_html = Html::parse_document(&output_html_string);

        assert_eq!(output_html.quirks_mode, QuirksMode::NoQuirks);
        let output_html_body_elements: Vec<ElementRef> = output_html
            .select(&Selector::parse("body").unwrap())
            .collect();
        assert_eq!(output_html_body_elements.len(), 1);
        let output_html_body = output_html_body_elements.get(0).unwrap();

        assert_eq!(
            correct_output_html_body.inner_html(),
            output_html_body.inner_html()
        );

        let output_html_link_elements: Vec<ElementRef> = output_html
            .select(&Selector::parse("link").unwrap())
            .collect();
        assert_eq!(output_html_link_elements.len(), 1);
        let output_html_link = output_html_link_elements.get(0).unwrap();
        let output_html_link_href = output_html_link.value().attr("href").unwrap();

        let style_file_path = output_directory_path.join(output_html_link_href.strip_prefix('/').unwrap());
        println!("{:?}", &style_file_path);
        assert!(style_file_path.exists());

        // TODO
    }
}
