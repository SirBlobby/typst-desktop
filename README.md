# Typst Desktop

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/sirblobby/typst-desktop)
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
- **Two views**: A file viewer for browsing folders, projects, and documents, and an editor view for writing.
- **Single files or projects**: Open a lone `.typ` file, or a project folder with its own `typst.toml`, chapters, bibliography, and assets.
- **Live preview**: The preview recompiles as you type. Saving is not required to see changes.
- **Editor**: Typst syntax highlighting, snippet autocompletion, a formatting toolbar, inline diagnostics, and optional `tinymist` language server support.
- **Cloud sync**: Link a project to a TypstDrive Space to push and pull changes, with three-way merge and a conflict resolver for files edited in both places.
- **Images and fonts**: Import files from anywhere on disk, or drag and drop them in. A shared asset library makes images and fonts available to every project.
- **Thumbnails**: Documents show their first compiled page; images show a preview.
- **Image viewer**: Open images full screen with zoom and folder navigation.

## Storage

By default the workspace lives at `~/typst` (for example, `/home/blob/typst`). Change it in Settings.

Inside the workspace:

| Path | Contents |
|---|---|
| `<workspace>/` | Your folders, projects, and documents. |
| `<workspace>/.assets/` | Shared images and fonts available to every project. |

Application data — settings, sync state, and the thumbnail cache — is kept in a SQLite database in the platform's app-data directory. Documents themselves are never stored there.

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

## Cloud Sync

Sync is optional. Without it the app is entirely local.

1. Open **Settings**, enter your TypstDrive server URL, and sign in.
2. In the file viewer, choose **Upload to cloud** on a project to create a Space from it.
3. Use **Sync** in the editor to pull remote changes and push local ones.

A project already in the cloud can be brought to another device from the **Cloud** tab in the file viewer.

### Conflicts

Sync pulls before it pushes. When a file changed both locally and in the cloud, a three-way merge is attempted against the version last synced. If the merge cannot be resolved automatically, a conflict resolver opens where you can keep either version or edit the merged result. Binary files cannot be merged and keep the cloud version.

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
