# Typst Desktop

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/sirblobby/typst-desktop)
[![License](https://img.shields.io/badge/license-Apache_2.0-green.svg)](LICENSE)
[![Typst Version](https://img.shields.io/badge/Typst-0.14.2-239dad?logo=typst&logoColor=white)](https://typst.app/)
[![Rust](https://img.shields.io/badge/Rust-1.82+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-5-ff3e00?logo=svelte)](https://kit.svelte.dev/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)

A desktop editor for Typst. Your documents stay as plain files on your own drive, and can optionally sync to a [TypstDrive](https://github.com/SirBlobby/TypstDrive) server so the same notes open on any device.

The Typst compiler is built into the app — there is nothing extra to install to write and export documents.

## Features

- **Local first**: Every document is an ordinary file in a folder you choose. Nothing is locked in a database, and any other editor can open the same files.
- **Two views**: A file viewer for browsing local and cloud files, and an editor view for writing.
- **Single files or projects**: Open a lone `.typ` file, or a project folder with its own `typst.toml`, chapters, bibliography, and assets.
- **Live preview**: The preview recompiles as you type, and follows whichever file you are editing rather than only the entrypoint. Saving is not required to see changes.
- **Editor**: Typst syntax highlighting, snippet autocompletion, a formatting toolbar, page settings, inline diagnostics, and optional `tinymist` language server support.
- **File explorer**: Folder selection, a right-click menu, drag and drop, rename, duplicate, move, and reveal in your file manager.
- **Cloud workspace**: Browse your TypstDrive folders, documents, spaces, and everything shared with you. Download what you want on this device and open it straight from the cloud view.
- **Sync**: Three-way merge with a conflict resolver, plus optional automatic sync on a timer.
- **Autosave**: Off by default, or every 5, 10, or 15 seconds.
- **Images and fonts**: Import from anywhere on disk, drag and drop, or pull from your TypstDrive assets. A searchable asset browser with previews sits in the editor toolbar.
- **Thumbnails**: Documents and projects show their first compiled page; images show a preview.
- **Image viewer**: Open images full screen with zoom and folder navigation.

## Storage

By default the workspace lives at `~/typst` (for example, `/home/blob/typst`). Change it in Settings.

Inside the workspace:

| Path | Contents |
|---|---|
| `<workspace>/` | Your folders, projects, and documents. |
| `<workspace>/.assets/` | Shared images and fonts available to every project. |

Application data — settings, sync state, and the thumbnail cache — is kept in a SQLite database in the platform's app-data directory. Documents themselves are never stored there.

## Settings

Settings are grouped into four sections.

| Section | Contents |
|---|---|
| Files | Workspace folder and autosave delay. |
| Appearance | Light or dark theme. |
| Account | TypstDrive server, sign in and out, automatic sync interval. |
| About | Version, Typst version, license, and links to the official Typst site. |

## Fonts and Images

Fonts and images work the same way as in TypstDrive. A file is available to the compiler by its name.

### Fonts

Import `.ttf`, `.otf`, `.ttc`, or `.otc` files through **Assets**, or place them in a folder beside your document. Typst Desktop reads the family name embedded in the file and registers every weight and style under it.

```typst
#set text(font: "JetBrains Mono")
```

The toolbar's font dropdown lists every family available to the open document, including the fonts bundled with the app. If a family is not listed, it has not been picked up, and the name in your document will not resolve.

### Images

Reference an image by its file name:

```typst
#image("logo.png", width: 50%)
```

Images in the shared asset library are available to every project. Images inside a project folder are available to that project and override an asset of the same name.

## Cloud Workspace

Cloud features are optional. Without an account the app is entirely local.

1. Open **Settings**, go to **Account**, enter your TypstDrive server URL, and sign in.
2. Switch the file viewer to **Cloud**.

The cloud view has two places to look. **My Drive** shows your folders, documents, spaces, and uploaded images and fonts. **Shared with me** shows everything other people have shared with you, along with the role you were given.

| Item | Action |
|---|---|
| Document | Download to this device, then Open to edit it. |
| Space | Download as a local project, then Open. |
| Image or font | Download into your shared asset library. |

Cards show whether an item is on this device and whether it has local changes waiting to go up. Downloaded documents are managed from the cloud view rather than appearing a second time under Local.

To publish a local project, choose **Upload to cloud** on it in the Local view. That creates a Space from its contents.

### Sync

Sync pulls before it pushes. Use **Sync** in the editor, or turn on automatic sync in **Settings** to run it on a timer.

Viewer access is read-only: a document shared with the viewer role can be downloaded and read, but pushing changes back is refused.

### Conflicts

When a file changed both locally and in the cloud, a three-way merge is attempted against the version last synced. If the merge cannot be resolved automatically, a conflict resolver opens where you can keep either version or edit the merged result. Binary files cannot be merged and keep the cloud version. Automatic sync pauses while conflicts are unresolved.

## Language Server

Typst Desktop can use [`tinymist`](https://github.com/Myriad-Dreamin/tinymist) for hover documentation, go-to-definition, and richer diagnostics. It is not bundled — install it and make sure it is on your `PATH`:

```bash
curl -L -o ~/.cargo/bin/tinymist \
  https://github.com/Myriad-Dreamin/tinymist/releases/latest/download/tinymist-linux-x64 \
  && chmod +x ~/.cargo/bin/tinymist
```

The editor header shows the language server status. Without `tinymist` the editor still has syntax highlighting, autocompletion, and compiler diagnostics.

## Development

The app compiles Typst from source, so clone the compiler into TypstDrive's `typst/` folder first — both projects share it:

```bash
git clone https://github.com/typst/typst.git ../typstdrive/typst
```

Then:

```bash
bun install
bun run tauri dev
```

To build a release bundle:

```bash
bun run tauri build
```

### Layout

| Path | Contents |
|---|---|
| `src/lib/components/` | Views, editor, modals. |
| `src/lib/ts/` | Tauri command bindings and app state. |
| `src-tauri/src/workspace.rs` | Workspace browsing and file access. |
| `src-tauri/src/compiler.rs` | Typst compilation and export. |
| `src-tauri/src/world.rs` | Font and file resolution for the compiler. |
| `src-tauri/src/sync.rs` | TypstDrive sync and merging. |
| `src-tauri/src/assets.rs` | Shared image and font library. |
| `src-tauri/src/thumbnails.rs` | Preview rendering and cache. |
| `src-tauri/src/lsp.rs` | Language server bridge. |
| `src-tauri/src/db.rs` | Local SQLite storage. |

## License

Apache License 2.0. See [LICENSE](LICENSE).
