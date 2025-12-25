# Graphite Browser

A simple, sleek, modern, minimalist web browser compiled to WebAssembly (WASM) for near-native performance. This browser can run inside another browser!

## Features

- **Tab Management**: Create, close, and drag tabs to reorder them
- **Dynamic Tab Sizing**: Tabs automatically shrink as more are added
- **Navigation Controls**: Back, forward, and reload buttons
- **URL Bar**: Enter URLs or search queries
- **Search Engine Selection**: Choose from Yahoo, Google, Bing, DuckDuckGo, or Brave
- **Proxy Server Support**: Configure a WebSocket proxy for enhanced browsing
- **Downloads Panel**: View and manage downloads
- **Dark Mode Toggle**: Button ready for dark mode implementation
- **Hover Effects**: Visual feedback with hover states on icons
- **Favicon Display**: Shows home icon for tabs
- **State Persistence**: Browser state is saved to local storage

## Project Structure

```
rust-wasm-browser/
├── browser/           # Rust WASM browser application
│   ├── Cargo.toml    # Rust dependencies
│   └── src/
│       └── lib.rs    # Main browser code
├── sandbox/          # Host environment for the browser
│   ├── index.html    # HTML host page
│   ├── styles.css    # Browser styling
│   └── pkg/          # Compiled WASM output
└── README.md
```

## Building

### Prerequisites

- Rust (1.70+)
- wasm-pack

### Build Commands

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the browser
cd browser
wasm-pack build --target web --out-dir ../sandbox/pkg
```

## Running

After building, serve the `sandbox` directory with any HTTP server:

```bash
cd sandbox
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

## Technology Stack

- **Rust**: Systems programming language for performance
- **Yew**: Rust framework for building web applications
- **wasm-bindgen**: Facilitating communication between Rust and JavaScript
- **web-sys**: Bindings to Web APIs
- **gloo**: Rust libraries for web development

## Screenshots

The browser includes:
- Tab bar with favicon, title, and close button
- Navigation controls (back, forward, reload)
- URL/search bar
- Toolbar with search, dark mode toggle, home, downloads, and settings
- Home page with search functionality
- Settings panel for search engine and proxy configuration
- Downloads panel for managing downloads
