#!/bin/bash

# Paperclip Installation Script
# Works on Linux and macOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed!"
        print_status "Please install Rust from: https://rustup.rs/"
        print_status "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed!"
        print_status "Please install Rust/Cargo from: https://rustup.rs/"
        exit 1
    fi
    
    print_success "Rust and Cargo are installed"
}

# Install paperclip
install_paperclip() {
    local os=$(detect_os)
    
    print_status "Building paperclip for $os..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_success "Build successful!"
        
        # Determine installation method
        if command -v cargo &> /dev/null; then
            print_status "Installing via cargo..."
            cargo install --path .
            print_success "Paperclip installed via cargo install"
        else
            # Fallback to manual installation
            print_status "Installing manually to ~/.local/bin..."
            mkdir -p ~/.local/bin
            cp target/release/paperclip ~/.local/bin/
            chmod +x ~/.local/bin/paperclip
            print_success "Paperclip installed to ~/.local/bin/paperclip"
        fi
    else
        print_error "Build failed!"
        exit 1
    fi
}

# Check PATH configuration
check_path() {
    local cargo_bin="$HOME/.cargo/bin"
    local local_bin="$HOME/.local/bin"
    local os=$(detect_os)
    
    # Check if cargo bin is in PATH
    if [[ ":$PATH:" != *":$cargo_bin:"* ]]; then
        print_warning "Cargo bin directory ($cargo_bin) is not in your PATH"
        
        case $os in
            "macos")
                print_status "Add this to your ~/.zshrc (or ~/.bash_profile):"
                echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
                print_status "Then run: source ~/.zshrc"
                ;;
            "linux")
                print_status "Add this to your ~/.bashrc or ~/.zshrc:"
                echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
                print_status "Then run: source ~/.bashrc (or ~/.zshrc)"
                ;;
        esac
        echo ""
    fi
    
    # Check if local bin is in PATH (fallback)
    if [[ ":$PATH:" != *":$local_bin:"* ]]; then
        print_status "Optional: Add ~/.local/bin to PATH as well:"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    fi
}

# Verify installation
verify_installation() {
    if command -v paperclip &> /dev/null; then
        print_success "Paperclip is installed and available in PATH!"
        print_status "Version: $(paperclip --version 2>/dev/null || echo 'v0.1.0')"
    else
        print_warning "Paperclip is installed but not found in PATH"
        print_status "You may need to restart your terminal or source your shell profile"
    fi
}

# Platform-specific setup
platform_setup() {
    local os=$(detect_os)
    
    case $os in
        "macos")
            print_status "macOS detected"
            # Check for Xcode Command Line Tools
            if ! xcode-select -p &> /dev/null; then
                print_warning "Xcode Command Line Tools not found"
                print_status "You may need to install them: xcode-select --install"
            fi
            
            # Check for Homebrew (optional)
            if command -v brew &> /dev/null; then
                print_status "Homebrew detected - you could also install via: brew install rust"
            fi
            ;;
        "linux")
            print_status "Linux detected"
            # Check for common build tools
            if ! command -v gcc &> /dev/null && ! command -v clang &> /dev/null; then
                print_warning "No C compiler found (gcc/clang)"
                print_status "You may need to install build-essential or development tools"
            fi
            ;;
    esac
}

# Main installation process
main() {
    echo "================================================"
    echo "           Paperclip Installation"
    echo "         Terminal Todo Manager"
    echo "================================================"
    echo ""
    
    # Platform setup
    platform_setup
    echo ""
    
    # Check prerequisites
    check_rust
    echo ""
    
    # Install
    install_paperclip
    echo ""
    
    # Check PATH
    check_path
    
    # Verify
    verify_installation
    
    echo ""
    echo "================================================"
    print_success "Installation complete!"
    echo "================================================"
    print_status "Features:"
    print_status "  • Multi-workspace support"
    print_status "  • Hierarchical todos with expansion/collapse"
    print_status "  • Tags and contexts (#tag @context)"
    print_status "  • Due dates and priorities"
    print_status "  • Notes and time tracking"
    print_status "  • Templates and recurrence patterns"
    print_status "  • Clean monochrome terminal UI"
    echo ""
    print_status "Quick start:"
    print_status "  1. Run: paperclip"
    print_status "  2. Navigate with j/k, select with Enter"
    print_status "  3. Press 'n' to create workspace"
    print_status "  4. Press 'i' to add todos"
    print_status "  5. Press '?' for complete help"
    echo ""
    print_status "Workspace controls:"
    print_status "  w: Switch workspace | n: New | d: Delete"
    echo ""
}

# Run main function
main "$@"
