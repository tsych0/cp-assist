# CP-Assist

CP-Assist is a powerful desktop application designed to supercharge your competitive programming workflow. It seamlessly integrates with tools like [Competitive Companion](https://github.com/jmerle/competitive-companion) to automate test case management, code execution, and dependency handling, letting you focus on solving problems.

## Table of Contents
- [App Preview](#app-preview)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Technical Details](#technical-details)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## App Preview

**Main Interface**
![Main application window showing problem details and test cases](https://github.com/tsych0/cp-assist/blob/main/cp-assist-shot.png?raw=true)

**Built-in Error Highlighting**
![Example of a compilation error within the app](https://github.com/tsych0/cp-assist/blob/main/compilation_error.png?raw=true)

**Demo Video**


https://github.com/user-attachments/assets/032702bf-5021-4046-a83d-9e7cec3047b4



## Features

- **Automated Problem Parsing**: Instantly import problem details and test cases from any supported online judge using the [Competitive Companion](https://github.com/jmerle/competitive-companion) browser extension.
- **Local Testing Environment**: Run your code against all test cases locally, with detailed feedback on status (Accepted, Wrong Answer, TLE), execution time, and memory usage.
- **Smart Templating System**: Use **Handlebars** templates to dynamically generate file names and structure your code, keeping your projects organized.
- **Intelligent Dependency Injection**: The app automatically detects which library files you are using and injects them as `mod`s at compile time, so your solution file stays clean.
- **Multi-language Support**: Comes with pre-configured settings for C++, Rust, Python, and more. Easily extendable through a simple `Languages.toml` file.
- **Seamless Editor Integration**: Opens your generated solution file directly in your favorite code editor (VS Code, Zed, Sublime, etc.).
- **One-Click Submission**: Submit your final solution to the judge using the [CP-Submit](https://github.com/tsycho/cp-submit) integration (optional).

## Installation

### Prerequisites
- [Competitive Companion](https://github.com/jmerle/competitive-companion) browser extension (configure it to use port **27121**).

### Release Downloads
Grab the latest version from the [**Releases Page**](https://github.com/tsych0/cp-assist/releases/latest).

- **Windows**: Download the `.exe` installer.
- **macOS (Apple Silicon)**: Download the `.dmg` file.
- **Linux**:
  - **Debian/Ubuntu**: Download the `.deb` package.
  - **Fedora/Red Hat**: Download the `.rpm` package.
  - **Arch Linux/Other**: Use the `.AppImage`. You may need to make it executable first:
    ```bash
    chmod +x cp-assist_*.AppImage
    ./cp-assist_*.AppImage
    ```

## Usage

1.  **Set Your Project Directory**: Launch the app and choose a folder where your solution files will be saved.
2.  **Import a Problem**: Navigate to a problem page on a site like Codeforces and click the Competitive Companion icon in your browser. The problem data will instantly appear in CP-Assist.
3.  **Create Solution File**: Click the **"Create File"** button. The app will:
    -   Generate a new file based on your `config.toml` template.
    -   Open the file in your configured editor.
4.  **Write Your Code**: Solve the problem in your editor. The app will watch for file changes.
5.  **Test and Iterate**: Every time you save the file, the app will automatically compile and run it against all test cases, giving you instant feedback.

## Configuration

CP-Assist is highly customizable via a `config.toml` file located in your app's configuration directory.

### `config.toml`

This file controls file generation, dependency management, and editor integration.

```toml
# Your name, used in file headers.
author = "Your Name"

# The command to open your editor.
editor = "code" # "code" for VS Code, "zed" for Zed, etc.

[toggle]
# Automatically create a file when a new problem is parsed.
create_file = true
# Automatically run tests whenever the solution file is saved.
run_on_save = true
# Automatically trigger a submission via CP-Submit on getting all Accepted verdicts.
submit_on_ac = false

[code]
# A Handlebars template for generating the solution filename.
# You can use variables like `title` and `url`.
filename = """
{{#with (regex_captures
  pattern="problemset/problem/(\\d+)/([A-Za-z0-9]+)"
  on=url) as |url_parts|}}
  {{#with (regex_captures
    pattern="^[A-Za-z0-9]+\\.\\s*(.+)$"
    on=../title) as |title_parts|}}
./src/bin/{{url_parts._1}}-{{to_lower_case url_parts._2}}-{{to_kebab_case title_parts._1}}.rs
  {{/with}}
{{/with}}
"""

# Path to your default code template.
template = "./templates/main.rs"

# A Handlebars template that assembles your final code for compilation.
# It injects used library files from the `[include]` map below.
modifier = """
{{{code}}}

{{#each lib_files}}
mod {{@key}} {
    {{{this}}}
}
{{/each}}
"""

# A regex to detect library usage (e.g., `use my_lib::...;`).
lib_check_regex = "use.*{{name}}(::|;)"

# Map of local library files/directories to be considered for injection.
[include]
cpio = "./src/cpio.rs"
utils = "./src/utils/" # You can include a whole directory
```

### `Languages.toml`
This file configures the compilers and commands for each language. You can edit it to add new languages or tweak compiler flags.

## Technical Details

- **Backend**: Rust with Tauri for a lightweight, cross-platform, and native experience.
- **Frontend**: React, TypeScript, and Mantine for a clean and responsive UI.
- **Templating**: Handlebars for powerful and flexible file/code generation.

## Development

Interested in contributing?

```bash
# Clone the repository
git clone https://github.com/tsych0/cp-assist.git
cd cp-assist

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

## Troubleshooting

- **Competitive Companion isn't working**: Ensure it is configured to connect to `http://localhost:27121`.
- **File not created**: Check the logs in the terminal (if running in dev mode) for errors related to your `config.toml` templates.
- **Compiler/Runtime Errors**: Make sure you have the necessary toolchains (e.g., `g++`, `rustc`, `python`) installed and available in your system's PATH.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

Made with ❤️ by [Ayush Biswas](https://github.com/tsych0)
