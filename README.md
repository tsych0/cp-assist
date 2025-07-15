# CP-Assist

CP-Assist is a comprehensive desktop application designed to streamline your competitive programming workflow, integrating with popular tools to automate test case management, code execution, and solution submission.

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

![alt text](https://github.com/veryshyjelly/cp-assist/blob/main/cp-assist-shot.png?raw=true)

https://github.com/user-attachments/assets/516eacd3-817e-4e87-bcbf-0817f914553f

## Features

### Test Case Management
- **Automated Capture**: Integrates with Competitive Companion browser extension to automatically import problem statements and test cases
- **Organized Interface**: View and manage multiple test cases in a clean, intuitive interface
- **Real-time Status**: Track verdict status (Accepted, Wrong Answer, Runtime Error) for each test case

### Code Execution & Judging
- **Local Testing**: Run your solutions against test cases without leaving the app
- **Performance Metrics**: Track execution time and memory usage for each test run
- **Verdict Generation**: Automatic comparison of expected and actual outputs

### Language Support
- **Multi-language Environment**: Built-in support for popular programming languages (C++, Python, Java, Rust, etc.)
- **Customizable Configurations**: Modify compiler flags and execution parameters via TOML configuration
- **Template Management**: Create and use language-specific templates for faster coding

### Submission System
- **Direct Integration**: Submit your solutions to online judges without switching applications
- **Status Tracking**: Monitor submission status via CP-Submit integration
- **History Management**: Keep track of all your previous submissions

## Installation

### Prerequisites
- [Competitive Companion](https://github.com/jmerle/competitive-companion) browser extension
- [CP-Submit](https://github.com/tsycho/cp-submit) for submission integration

### Linux
- **AppImage**: [Download](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist_0.4.0_amd64.deb)
  ```bash
  # Making AppImage executable
  chmod +x cp-assist_0.4.0_amd64.AppImage
  ./cp-assist_0.4.0_amd64.AppImage
  ```
- **Arch Linux**: [.tar file](https://github.com/tsych0/cp-assist/archive/refs/tags/v0.4.0.tar.gz)
- **Debian/Ubuntu**: [.deb file](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist_0.4.0_amd64.deb)
- **Red Hat/Fedora**: [.rpm file](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist-0.4.0-1.x86_64.rpm)

### Windows
- **Setup EXE**: [Download](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist_0.4.0_x64-setup.exe)

### Mac
- **DMG File**: [Download](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist_0.4.0_aarch64.dmg)
- **App zip**: [Download](https://github.com/tsych0/cp-assist/releases/download/v0.4.0/cp-assist.app.zip)

## Usage

### Getting Started
1. **Create or Open Project**: Start CP-Assist and create a new project or open an existing directory
2. **Import Problems**:
   - Navigate to a problem page on a supported online judge (like Codeforces)
   - Click the Competitive Companion browser extension
   - The problem details and test cases will be automatically imported into CP-Assist

### Solving Problems
1. **Select Language**: Choose your preferred programming language from the dropdown menu
2. **Write Solution**: Click "Create File" to generate a new solution file with appropriate templates
3. **Test Solution**:
   - Click "Run" to execute your solution against all test cases
   - View detailed results including execution time and memory usage
   - Examine differences between expected and actual outputs

### Submitting Solutions
1. **Review Solution**: Ensure all test cases pass locally
2. **Submit**: Click "Submit" to send your solution directly to the online judge
3. **View Status**: The submission status will be reported via CP-Submit integration

## Configuration

CP-Assist uses a TOML configuration file system that can be customized to fit your workflow.

### Language Configuration
Edit the `Languages.toml` file to:
- Add or modify programming language support
- Configure compiler flags and execution parameters
- Set up custom templates for each language

### Editor Preferences
Configure your preferred code editor in `config.toml`:
```toml
[code]
editor = "code"  # For VS Code
# Other options: "sublime", "vim", etc.
```

### File Naming and Templates
Customize how files are named and templated:
```toml
[code]
filename = "..."  # JavaScript function to generate filenames
template = "./templates/default.rs"  # Path to template file
```

### Sample config.toml
```toml
author = "Ayush Biswas"

editor = "zeditor"

[code]
filename = '''
function filename(title, url) {
  const urlMatch = url.match(/problemset\/problem\/(\d+)\/([A-Za-z0-9]+)/i);
  if (!urlMatch) throw new Error("Invalid Codeforces problem URL");
  const contestId = urlMatch[1];
  const problemIndex = urlMatch[2].toLowerCase();

  // Extract problem index and actual title from title string
  const titleMatch = title.match(/^([A-Za-z0-9]+)\.\s*(.+)$/);
  if (!titleMatch) throw new Error("Title format should be like 'A. Problem Title'");
  const problemTitle = titleMatch[2];

  // Format title: lowercase, words separated by hyphens
  const formattedTitle = problemTitle
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-") // Replace non-alphanumeric with hyphen
    .replace(/^-+|-+$/g, "")     // Trim leading/trailing hyphens
    .replace(/-+/g, "-");        // Collapse multiple hyphens

  return `./src/bin/${contestId}-${problemIndex}-${formattedTitle}.rs`;
}
'''
template = "./src/main.rs"
modifier = '''
function modify(code, lib_files) {
    return `
${code}

mod cpio {
    ${lib_files.cpio}
}

mod itertools {
    ${lib_files.itertools}
}`;
}
'''

[include]
cpio = "./src/cpio.rs"
itertools = "./src/itertools.rs"
```

## Technical Details

CP-Assist is built with modern technologies:

- **Frontend**: React with TypeScript, Mantine UI components, and Tailwind CSS
- **Backend**: Rust with Tauri framework for native performance
- **Communication**: Uses Tauri's API for seamless frontend-backend interaction
- **Process Management**: Custom Rust implementation for safe code execution and resource limiting

## Development

### Project Setup
```bash
# Clone the repository
git clone https://github.com/veryshyjelly/cp-assist.git
cd cp-assist

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### Building from Source
```bash
# Build for production
pnpm tauri build
```

## Troubleshooting

### Common Issues

- **Competitive Companion Not Working**: Ensure it's properly configured to use port 27121
- **Submission Failures**: Check that CP-Submit is correctly installed and configured
- **Execution Errors**: Verify you have the required compilers/interpreters installed for your chosen languages

## License

CP-Assist is released under the MIT License. See [LICENSE](LICENSE) file for details.

---

Made with ❤️ by [Ayush Biswas](https://github.com/veryshyjelly)
