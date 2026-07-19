# Typst Desktop 1.0.0

The first release. A desktop editor for Typst with the compiler built in, so there is nothing else to install.

## Features

### Writing

- Live preview that recompiles as you type, with no need to save
- The preview follows the file you are editing, not only the entrypoint
- Syntax highlighting, snippet autocompletion, and inline diagnostics
- Formatting toolbar and page settings
- Optional `tinymist` language server support for hover documentation and go to definition
- Export to PDF

### Files

- Every document is an ordinary file on your drive, readable by any other editor
- Open a single `.typ` file or a project folder with chapters, bibliography, and assets
- File explorer with a right-click menu, drag and drop, rename, duplicate, move, and reveal in your file manager
- Thumbnails showing the first compiled page, and previews for images
- Full screen image viewer with zoom and folder navigation
- Autosave, off by default or every 5, 10, or 15 seconds

### Fonts and images

- Import from anywhere on disk, or drag and drop into the workspace
- Searchable asset browser with previews in the editor toolbar
- Shared asset library available to every project, overridable per project
- `.ttf`, `.otf`, `.ttc`, and `.otc` fonts registered by their embedded family name

### Cloud

Optional. Without an account the app is entirely local.

- Browse your TypstDrive folders, documents, spaces, and anything shared with you
- Download to this device and open straight from the cloud view
- Three-way merge with a conflict resolver when a file changed in both places
- Automatic sync on a timer, paused while conflicts are unresolved
- Upload a local project to the cloud as a space
- Role permissions honoured, with viewer access kept read-only

### Appearance

- Light and dark themes
- Settings grouped into Files, Appearance, Account, and About

## Install

Packages for Debian, Fedora, and Arch are in [the README](README.md#install).

## Known limitations

- Only Linux packages are built automatically. Windows and macOS have to be built on their own platforms.
- macOS builds are unsigned, so Gatekeeper blocks them on first launch.
- The language server is not bundled. Install `tinymist` separately for its features.
