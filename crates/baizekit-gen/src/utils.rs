use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub fn render(output_name: String, rendered: String, output_dir: &Option<String>) {
    match output_dir {
        None => {
            println!("============ start generate {} ==========", output_name);
            println!("{}", rendered);
            println!("============ {} generated ==========\n\n\n", output_name);
        }
        Some(output_dir) => {
            create_dir_all(&output_dir).unwrap();
            let output_path = PathBuf::from(&output_dir).join(output_name);
            fs::write(&output_path, rendered).unwrap();
            println!("✅ 输出到: {}", output_path.display());
        }
    }
}
