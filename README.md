# rust-ingest

`rust-ingest` is a command-line tool that generates a comprehensive text digest of a code repository. It intelligently traverses a directory, bundling all relevant source code into a single text file. This is particularly useful for providing context to Large Language Models (LLMs) or for creating a complete snapshot of a project's source for archival or review.

The tool is smart about what it includes, automatically respecting `.gitignore` rules and skipping common binary files, build artifacts, and dependency directories.

## Features

-   **Intelligent Filtering**: Automatically respects `.gitignore`, `.ignore`, and other standard ignore files.
-   **Default Ignores**: Comes with a built-in list of common directories (`node_modules`, `target`, etc.) and files (`.DS_Store`, `package-lock.json`) to exclude.
-   **Customizable Patterns**: Use glob patterns to precisely `--include` or `--exclude` files and directories.
-   **Content-Aware**: Includes a file's path in the directory tree but can exclude its content from the digest (e.g., for images, archives, or large files).
-   **Size Limiting**: Exclude content from files that exceed a configurable size limit with the `--max-size` flag.
-   **Fast**: Built in Rust for high performance, even on large repositories.

## Installation

You can install `rust-ingest` directly from source using Cargo.

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/your-username/rust-ingest.git
    cd rust-ingest
    ```

2.  **Install the binary:**
    ```sh
    cargo install --path .
    ```

This command compiles and installs the `rust-ingest` executable in your Cargo binary path (`~/.cargo/bin`), making it available from anywhere in your terminal.

## Usage

Once installed, you can run the tool using the `rust-ingest` command.

#### Basic Usage

To run on the current directory and create a `digest.txt` file:
```sh
rust-ingest
```

To analyze a different repository:
```sh
rust-ingest /path/to/another/repo
```

#### Common Examples

**Specify a different output file:**
```sh
rust-ingest . --output my-project-digest.txt
```

**Include only specific file types (e.g., Rust and TOML files):**
```sh
rust-ingest . --include "*.rs" --include "*.toml"
```

**Exclude a specific directory and all log files:**
```sh
rust-ingest . --exclude "docs/" --exclude "*.log"
```

**Increase the file size limit for content inclusion to 200 KB:**
```sh
rust-ingest . --max-size 200
```

### Command-Line Options

Here is the complete list of options available:

```sh
$ rust-ingest --help
Generate a directory content digest, intelligently excluding non-source files.

Usage: rust-ingest [OPTIONS] [PATH]

Arguments:
  [PATH]  The root directory to process [default: .]

Options:
  -i, --include <INCLUDE>  Glob patterns for files to include. If used, only matching files are included
  -e, --exclude <EXCLUDE>  Additional glob patterns for files or directories to exclude
      --max-size <MAX_SIZE>
          Maximum file size in KB for content inclusion [default: 100]
  -o, --output <OUTPUT>
          Output file name [default: digest.txt]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Development

To build and run this project from source for development purposes:

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/your-username/rust-ingest.git
    cd rust-ingest
    ```

2.  **Build and run with Cargo:**
    ```sh
    # This runs the tool on itself
    cargo run -- . --output self-digest.txt
    ```

## License
```
This project is licensed under the MIT License.
```
