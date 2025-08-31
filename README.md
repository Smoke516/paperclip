# Paperclip 📎

A powerful terminal-based todo list manager with workspaces, hierarchical todos, tags, contexts, and advanced features.

![Terminal Interface](https://img.shields.io/badge/interface-terminal-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey)

## Features

### Core Features
- **🗂️ Multi-workspace support** - Organize todos by project, context, or category
- **📋 Hierarchical todos** - Create subtasks and organize todos in trees
- **🏷️ Tags and contexts** - Use `#tags` and `@contexts` for organization
- **📅 Due dates** - Smart date parsing (`due:today`, `due:2024-12-25`, etc.)
- **⭐ Priority levels** - 0-5 priority scale with visual indicators
- **📝 Notes** - Add detailed notes to any todo
- **⏱️ Time tracking** - Track time spent on todos
- **🔁 Recurring todos** - Daily, weekly, monthly, yearly, or custom patterns
- **📋 Templates** - Create reusable todo templates

### Interface
- **Clean monochrome UI** - Works in any terminal
- **Vim-like navigation** - `j/k` for movement, intuitive keybindings
- **Tokyo Night theme** - Beautiful color scheme
- **Responsive design** - Adapts to terminal size

## Installation

### Prerequisites
- **Rust** (1.70+) - Install from [rustup.rs](https://rustup.rs/)

### macOS Installation

#### Method 1: Using the install script (Recommended)
```bash
# Clone the repository
git clone https://github.com/Smoke516/paperclip.git
cd paperclip

# Run the installation script
chmod +x install.sh
./install.sh
```

#### Method 2: Manual installation with Cargo
```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.zshrc

# Clone and install
git clone https://github.com/Smoke516/paperclip.git
cd paperclip
cargo install --path .
```

#### Method 3: Direct from crates.io (when published)
```bash
cargo install paperclip
```

### Linux Installation

#### Method 1: Using the install script (Recommended)
```bash
# Clone the repository
git clone https://github.com/Smoke516/paperclip.git
cd paperclip

# Run the installation script
chmod +x install.sh
./install.sh
```

#### Method 2: Manual installation
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc

# Install build tools (Ubuntu/Debian)
sudo apt update
sudo apt install build-essential

# Clone and install
git clone https://github.com/Smoke516/paperclip.git
cd paperclip
cargo install --path .
```

### PATH Configuration

After installation, make sure `~/.cargo/bin` is in your PATH:

**macOS (zsh):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Linux (bash):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Linux (zsh):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Quick Start

1. **Launch the app:**
   ```bash
   paperclip
   ```

2. **First time setup:**
   - App starts in workspace selection mode
   - Press `Enter` to select the default "Personal" workspace
   - Or press `n` to create a new workspace

3. **Add your first todo:**
   - Press `i` to enter insert mode
   - Type: `Fix bug #urgent @work due:today`
   - Press `Enter` to save

4. **Navigate and manage:**
   - Use `j/k` or arrow keys to navigate
   - Press `Space` to mark todos complete
   - Press `a` to add subtasks
   - Press `?` for full help

## Usage

### Basic Operations
| Key | Action |
|-----|--------|
| `i` | Add new todo |
| `a` | Add child todo (subtask) |
| `e` | Edit selected todo |
| `Space` | Toggle todo completion |
| `d` | Delete selected todo |
| `D` | Delete todo and all children |
| `j/k` or `↓/↑` | Navigate up/down |
| `g/G` | Go to top/bottom |
| `Enter` | Expand/collapse todo |

### Workspaces
| Key | Action |
|-----|--------|
| `w` | Open workspace selection |
| `n` | Create new workspace (in workspace selection) |
| `d` | Delete workspace (in workspace selection) |
| `Enter` | Select workspace (in workspace selection) |
| `Esc` | Cancel workspace selection |

### Search and Filtering
| Key | Action |
|-----|--------|
| `/` | Search todos |
| `#` | Filter by tag |
| `@` | Filter by context |
| `!` | Cycle due date filters |
| `v` | Cycle view mode (all/pending/completed) |
| `Esc` | Clear filters |

### Advanced Features
| Key | Action |
|-----|--------|
| `n` | Edit notes for selected todo |
| `V` | View notes (read-only) |
| `t` | Toggle timer for selected todo |
| `T` | Apply template |
| `r` | Set recurrence pattern |
| `+/-` | Increase/decrease priority |

### Todo Format
Create rich todos with inline metadata:
```
Fix authentication bug #urgent @backend due:friday
```

This creates a todo with:
- Description: "Fix authentication bug"
- Tag: `urgent`
- Context: `backend`
- Due date: Next Friday

### Visual Indicators
- `○` Pending | `◐` In Progress | `●` Completed
- `!` Overdue | `▼▶` Expandable | `[!]` Priority
- `#tag` Tags (cyan) | `@context` Contexts (orange)
- `[N]` Has notes | `[today]` Due dates


## File Storage

Paperclip stores data in platform-appropriate locations:
- **macOS**: `~/Library/Application Support/paperclip/`
- **Linux**: `~/.local/share/paperclip/`

Data is stored in JSON format and automatically saved.


## Building from Source

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git

### Build
```bash
git clone https://github.com/seawn/paperclip.git
cd paperclip
cargo build --release
```

### Run without installing
```bash
cargo run
```

## Troubleshooting

### macOS Issues
- **"cannot be opened because the developer cannot be verified"**: 
  - Right-click the binary and select "Open" or run `xattr -d com.apple.quarantine ~/.cargo/bin/paperclip`
- **Missing Xcode Command Line Tools**: Run `xcode-select --install`

### Linux Issues
- **Missing build tools**: Install `build-essential` on Ubuntu/Debian or equivalent on your distro
- **Terminal compatibility**: Works best with modern terminals that support Unicode

### General Issues
- **Command not found**: Ensure `~/.cargo/bin` is in your PATH
- **Garbled display**: Update your terminal or try a different terminal emulator
- **Permission denied**: Run `chmod +x ~/.cargo/bin/paperclip`

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the terminal UI
- Inspired by todo.txt and taskwarrior
- Tokyo Night color theme

---

**Happy organizing! 📎✨**
