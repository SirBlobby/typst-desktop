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

## Install

Packages are attached to each [release](https://github.com/sirblobby/typst-desktop/releases). Set the version you want first:

```bash
VERSION=1.0.0
BASE=https://github.com/sirblobby/typst-desktop/releases/download/v$VERSION
```

### Debian and Ubuntu

```bash
wget "$BASE/typst-desktop_${VERSION}_amd64.deb"
sudo apt install "./typst-desktop_${VERSION}_amd64.deb"
```

Installing with `apt` rather than `dpkg -i` pulls in `libwebkit2gtk-4.1-0` and `libgtk-3-0` for you. Debian 12 or Ubuntu 22.04 and newer carry a new enough WebKit.

Remove it with `sudo apt remove typst-desktop`.

### Fedora

```bash
wget "$BASE/typst-desktop-${VERSION}-1.x86_64.rpm"
sudo dnf install "./typst-desktop-${VERSION}-1.x86_64.rpm"
```

Remove it with `sudo dnf remove typst-desktop`.

### Arch

There is no package in the AUR. Use the AppImage, which needs FUSE:

```bash
sudo pacman -S fuse2
wget "$BASE/typst-desktop_${VERSION}_amd64.AppImage"
chmod +x "typst-desktop_${VERSION}_amd64.AppImage"
./typst-desktop_${VERSION}_amd64.AppImage
```

To keep it on your `PATH`:

```bash
sudo install -Dm755 "typst-desktop_${VERSION}_amd64.AppImage" /usr/local/bin/typst-desktop
```

Remove it with `sudo rm /usr/local/bin/typst-desktop`.

The AppImage runs on any distribution, so it also works as a fallback on Debian or Fedora. Building from source is covered under [Development](#development).

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

To build a release bundle for your own platform:

```bash
bun run tauri build
```

On Linux, use `bun run build:linux` instead. It is the same command with `NO_STRIP=1` set, which the AppImage bundler needs on a rolling-release distribution. See [Platform notes](#platform-notes).

## Distribution

Workflows live in `.gitea/workflows` and run on Gitea Actions.

| Workflow | Trigger | Output |
|---|---|---|
| `ci.yml` | push, pull requests | Type check, frontend build, backend check. |
| `build.yml` | `v*` tags, manual | Linux packages as workflow artifacts, and on a tag, a draft GitHub release. |

| Platform | Artifacts |
|---|---|
| Linux | `.deb`, `.rpm`, `.AppImage` |

Builds run on a Linux runner, so Linux packages are the only artifacts produced. Windows and macOS installers have to be built on their own platforms and attached to the release by hand.

Publishing to GitHub needs one repository secret:

| Secret | Value |
|---|---|
| `RELEASE_TOKEN` | A classic GitHub personal access token with the `repo` scope. |

To cut a release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The release is drafted rather than published, so you can add other platforms and review the artifacts before making it public.

### Platform notes

`.msi` packages can only be built on Windows, because the WiX Toolset is Windows-only. Building on the target platform is the supported path for both Windows and macOS.

The Windows installers download the WebView2 bootstrapper when the runtime is missing. Windows 10 (April 2018 or later) and Windows 11 already ship it, so this only affects older systems. The NSIS installer asks whether to install for the current user or the whole machine.

macOS builds are unsigned. Gatekeeper blocks unsigned apps on first launch, so open the app once with right-click then **Open**, or run `xattr -dr com.apple.quarantine "/Applications/Typst Desktop.app"` after installing.

### Building locally for another platform

Building on the target platform is the supported path. The Typst compiler is a path dependency, so any machine or runner needs it checked out beside this repository:

```bash
git clone https://github.com/typst/typst.git ../typstdrive/typst
git -C ../typstdrive/typst checkout 44b3f78ed37fedea75e911dde2269ef86c45316f
```

Linux builds also need the WebKit and GTK development packages:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
```

On rolling-release distributions such as Arch, the AppImage step fails with `failed to run linuxdeploy`. The `strip` bundled inside `linuxdeploy` predates the `.relr.dyn` relocation section that a current toolchain emits, so it rejects every system library it is asked to strip. Setting `NO_STRIP=1` skips that pass, which is what `build:linux` does:

```bash
bun run build:linux
```

The `.deb` and `.rpm` targets are unaffected, so a build that produces those two and then fails is almost always this.

## License

Apache License 2.0. See [LICENSE](LICENSE).
