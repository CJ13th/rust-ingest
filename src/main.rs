use anyhow::{Context, Result};
use clap::Parser;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::HashSet;


/// Generate a directory content digest, intelligently excluding non-source files.
#[derive(Parser, Debug)]
#[clap(name = "rust-ingest", version, about)]
struct Args {
    /// The root directory to process.
    #[clap(default_value = ".")]
    path: PathBuf,

    /// Glob patterns for files to include. If used, only matching files are included.
    #[clap(long, short)]
    include: Vec<String>,

    /// Additional glob patterns for files or directories to exclude.
    #[clap(long, short)]
    exclude: Vec<String>,

    /// Maximum file size in KB for content inclusion.
    #[clap(long, default_value_t = 100)]
    max_size: u64,

    /// Output file name.
    #[clap(long, short, default_value = "digest.txt")]
    output: String,
}

// --- Configuration: Default items to ignore ---
static DEFAULT_IGNORED_DIRS: &[&str] = &[
    ".git/", ".github/", ".vscode/", ".idea/", "venv/", ".env/", "node_modules/",
    ".next/", "out/", "__pycache__/", "target/", "pkg/", "build/", "dist/", "coverage/",
];

static DEFAULT_IGNORED_FILES: &[&str] = &[
    "pnpm-lock.yaml", "package-lock.json", "yarn.lock", "Cargo.lock",
    ".tsbuildinfo", ".DS_Store", "components.json", "biome.json", "next-env.d.ts",
    ".gitignore", ".prettierrc.json", "LICENSE", ".nvmrc", ".npmrc",
    ".eslintrc.json", ".prettierignore", "vercel.json",
];

static DEFAULT_EXCLUDED_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".svg",
    ".woff", ".woff2", ".ttf", ".eot", ".otf",
    ".zip", ".gz", ".tar", ".rar", ".7z", ".pack",
    ".wasm", ".dll", ".exe", ".so", ".a", ".lib", ".bin", ".o", ".pdf",
];

fn main() -> Result<()> {
    let args = Args::parse();
    let root = fs::canonicalize(&args.path)
        .with_context(|| format!("Failed to find or access path: {}", args.path.display()))?;

    if !root.is_dir() {
        anyhow::bail!("Provided path '{}' is not a directory.", root.display());
    }

    let mut override_builder = OverrideBuilder::new(&root);
    for pattern in DEFAULT_IGNORED_DIRS.iter().chain(DEFAULT_IGNORED_FILES) {
        override_builder.add(&format!("!{}", pattern))?;
    }
    override_builder.add(&format!("!{}", args.output))?;
    for pattern in &args.exclude {
        override_builder.add(&format!("!{}", pattern))?;
    }
    if !args.include.is_empty() {
        for pattern in &args.include {
            override_builder.add(pattern)?;
        }
    }
    let overrides = override_builder.build()?;
    let walker = WalkBuilder::new(&root)
        .standard_filters(true)
        .overrides(overrides)
        .build();

    println!("Discovering files...");

    let mut tree_files = Vec::new();
    let mut content_files = Vec::new();
    let max_size_bytes = args.max_size * 1024;
    let excluded_extensions: HashSet<&str> = DEFAULT_EXCLUDED_EXTENSIONS.iter().cloned().collect();

    for result in walker {
        let entry = result.context("Failed to process a directory entry")?;
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(&root).unwrap().to_path_buf();
            tree_files.push(relative_path.clone());

            if let Some(ext_os) = path.extension() {
                let ext = format!(".{}", ext_os.to_string_lossy().to_lowercase());
                if excluded_extensions.contains(ext.as_str()) {
                    println!("  -> Skipping content for excluded extension: {}", relative_path.display());
                    continue;
                }
            }

            if entry.metadata()?.len() > max_size_bytes {
                println!("  -> Skipping content for large file: {} (>{}KB)", relative_path.display(), args.max_size);
                continue;
            }
            content_files.push(relative_path);
        }
    }
    
    tree_files.sort();
    content_files.sort();

    println!("Found {} files for tree, {} files for content.", tree_files.len(), content_files.len());

    if tree_files.is_empty() {
        println!("No files to include based on current criteria. Exiting.");
        return Ok(());
    }

    println!("Generating directory tree...");
    let tree_structure = generate_tree(&root, &tree_files)?;

    println!("Reading and concatenating {} files...", content_files.len());
    let mut concatenated_content = String::new();
    for file_path in &content_files {
        concatenated_content.push_str(&"=".repeat(60));
        concatenated_content.push('\n');
        concatenated_content.push_str(&format!("FILE: {}\n", file_path.display()));
        concatenated_content.push_str(&"=".repeat(60));
        concatenated_content.push('\n');
        match fs::read_to_string(root.join(file_path)) {
            Ok(contents) => concatenated_content.push_str(contents.trim()),
            Err(e) => concatenated_content.push_str(&format!("[Could not read file: {}]", e)),
        }
        concatenated_content.push_str("\n\n\n");
    }
    
    println!("Writing output to {}...", args.output);
    let mut output_file = File::create(&args.output)
        .with_context(|| format!("Failed to create output file '{}'", args.output))?;
    
    writeln!(output_file, "Directory structure:")?;
    writeln!(output_file, "{}", tree_structure)?;
    writeln!(output_file, "\n\nFiles Content:\n")?;
    write!(output_file, "{}", concatenated_content)?;

    println!("All done. Digest saved to {}", args.output);
    Ok(())
}

fn generate_tree(root: &Path, included_files: &[PathBuf]) -> Result<String> {
    // A TreeNode holds a map of its children's names to the children nodes.
    #[derive(Default)]
    struct TreeNode {
        children: BTreeMap<String, TreeNode>,
    }

    let mut root_node = TreeNode::default();

    // Build the tree structure from the flat list of paths.
    for path in included_files {
        let mut current_node = &mut root_node;
        for component in path.components() {
            let part = component.as_os_str().to_string_lossy().to_string();
            current_node = current_node.children.entry(part).or_default();
        }
    }

    // A recursive helper function to build the string representation from the tree structure.
    fn build_string_recursive(node: &TreeNode, prefix: &str, output: &mut String) {
        let mut entries: Vec<_> = node.children.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0)); // Sort entries lexically.
        
        for (i, (name, child_node)) in entries.iter().enumerate() {
            let connector = if i == entries.len() - 1 { "└── " } else { "├── " };
            output.push_str(&format!("{}{}{}\n", prefix, connector, name));
            
            // Recurse if the child has its own children (i.e., it's a directory).
            if !child_node.children.is_empty() {
                let new_prefix = format!("{}{}", prefix, if i == entries.len() - 1 { "    " } else { "│   " });
                build_string_recursive(child_node, &new_prefix, output);
            }
        }
    }

    let root_name = root.file_name().unwrap().to_string_lossy();
    let mut tree_string = format!("└── {}/\n", root_name);
    build_string_recursive(&root_node, "    ", &mut tree_string);

    Ok(tree_string)
}