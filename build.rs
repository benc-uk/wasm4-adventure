use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct StoryYaml {
    sections: Vec<SectionYaml>,
}

#[derive(Debug, Deserialize)]
struct OptionYaml {
    label: String,
    target_section: String,
}

#[derive(Debug, Deserialize)]
struct SectionYaml {
    id: String,
    title: String,
    description: String,
    options: Vec<OptionYaml>,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    println!("### build.rs: OUTDIR={}", out_dir.to_string_lossy());
    let dest_path = Path::new(&out_dir).join("story.rs");

    let yaml = fs::read_to_string("data/story.yaml").expect("Could not read story.yaml");
    let story: StoryYaml = serde_yaml_ng::from_str(&yaml).expect("Invalid story.yaml");

    println!("### build.rs: YAML story file loaded");

    // First assign section ids
    let mut section_ids: HashMap<&str, usize> = HashMap::new();
    for (index, section) in story.sections.iter().enumerate() {
        assert!(
            section_ids.insert(section.id.as_str(), index).is_none(),
            "Duplicate section ID: {}",
            section.id
        );
        println!("### build.rs: Section: {} -> {}", section.id, index)
    }

    let mut generated = String::from(
        "pub fn load_story() { 
    let mut section: Section;
    unsafe {
    ",
    );

    for section in &story.sections {
        let title = &section.title;
        let description = &section.description;
        println!("### build.rs: Parsing Section: {}", title);

        let id = section_ids.get(&section.id.as_str()).unwrap();

        let mut options_vec = String::from("");
        for option in &section.options {
            let tid = section_ids.get(&option.target_section.as_str()).unwrap();
            let label = &option.label;
            let opt_str = format!(
                "Option {{ 
                label: {label:?},
                target_section: {tid:?},
            }},"
            );
            options_vec.push_str(&opt_str);
        }

        let sec_rust = format!(
            "section = Section {{
            id: {id:?},
            title: {title:?},
            description: {description:?},
            options: vec![{options_vec}],
        }};
        _ = GAME.sections.insert({id:?}, section);\n\n"
        );

        generated.push_str(&sec_rust);
    }

    generated.push_str("}\n}");
    fs::write(&dest_path, generated).unwrap();

    // Re-run if either source story or
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=data/story.yaml");
}
