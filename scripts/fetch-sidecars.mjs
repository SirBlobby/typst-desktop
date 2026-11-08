import { execFileSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const sidecars = [
  {
    name: "pandoc",
    version: "3.6.4",
    archiveByTarget: {
      "x86_64-unknown-linux-gnu": "linux-amd64.tar.gz",
      "aarch64-unknown-linux-gnu": "linux-arm64.tar.gz",
      "x86_64-apple-darwin": "x86_64-macOS.zip",
      "aarch64-apple-darwin": "arm64-macOS.zip",
      "x86_64-pc-windows-msvc": "windows-x86_64.zip",
    },
    archiveUrl: (version, suffix) => `https://github.com/jgm/pandoc/releases/download/${version}/pandoc-${version}-${suffix}`,
    licenseNames: ["COPYRIGHT", "COPYRIGHT.txt", "COPYING.md", "COPYING.txt"],
    licenseFile: "pandoc-COPYRIGHT.txt",
    fallbackLicense: (version) => `Pandoc ${version} is licensed under the GNU GPL, version 2 or later.\nSource code: https://github.com/jgm/pandoc\n`,
  },
  {
    name: "tinymist",
    version: "v0.15.12",
    archiveByTarget: {
      "x86_64-unknown-linux-gnu": "x86_64-unknown-linux-musl.tar.gz",
      "aarch64-unknown-linux-gnu": "aarch64-unknown-linux-musl.tar.gz",
      "x86_64-apple-darwin": "x86_64-apple-darwin.tar.gz",
      "aarch64-apple-darwin": "aarch64-apple-darwin.tar.gz",
      "x86_64-pc-windows-msvc": "x86_64-pc-windows-msvc.zip",
    },
    archiveUrl: (version, suffix) => `https://github.com/Myriad-Dreamin/tinymist/releases/download/${version}/tinymist-${suffix}`,
    licenseNames: ["LICENSE", "LICENSE.txt", "LICENSE.md"],
    licenseFile: "tinymist-LICENSE.txt",
    fallbackLicense: (version) => `Tinymist ${version} is licensed under the Apache License, version 2.0.\nSource code: https://github.com/Myriad-Dreamin/tinymist\n`,
  },
];

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const binariesDir = join(projectRoot, "src-tauri", "binaries");

function hostTargetTriple() {
  if (process.env.TAURI_ENV_TARGET_TRIPLE) return process.env.TAURI_ENV_TARGET_TRIPLE;
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const hostLine = output.split("\n").find((line) => line.startsWith("host:"));
  if (!hostLine) throw new Error("Could not determine the Rust target triple");
  return hostLine.replace("host:", "").trim();
}

function tarProgram() {
  if (process.platform !== "win32") return "tar";
  const systemRoot = process.env.SystemRoot || "C:\\Windows";
  return join(systemRoot, "System32", "tar.exe");
}

function findFile(directory, names) {
  for (const entry of readdirSync(directory)) {
    const fullPath = join(directory, entry);
    if (statSync(fullPath).isDirectory()) {
      const found = findFile(fullPath, names);
      if (found) return found;
    } else if (names.includes(entry)) {
      return fullPath;
    }
  }
  return null;
}

async function download(url, destination) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Download failed (${response.status}): ${url}`);
  writeFileSync(destination, Buffer.from(await response.arrayBuffer()));
}

async function fetchSidecar(sidecar, target) {
  const archiveSuffix = sidecar.archiveByTarget[target];
  if (!archiveSuffix) throw new Error(`No ${sidecar.name} build is configured for ${target}`);

  const isWindows = target.includes("windows");
  const executableName = isWindows ? `${sidecar.name}.exe` : sidecar.name;
  const sidecarPath = join(binariesDir, `${sidecar.name}-${target}${isWindows ? ".exe" : ""}`);
  const versionMarker = join(binariesDir, `${sidecar.name}-version.txt`);
  const markerValue = `${sidecar.version} ${target}`;

  if (existsSync(sidecarPath) && existsSync(versionMarker) && readFileSync(versionMarker, "utf8").trim() === markerValue) {
    return;
  }

  const url = sidecar.archiveUrl(sidecar.version, archiveSuffix);
  const workDir = join(tmpdir(), `typst-desktop-${sidecar.name}-${process.pid}`);
  const archivePath = join(workDir, url.split("/").pop());

  mkdirSync(workDir, { recursive: true });

  try {
    console.log(`Downloading ${sidecar.name} ${sidecar.version} for ${target}`);
    await download(url, archivePath);
    execFileSync(tarProgram(), ["-xf", archivePath, "-C", workDir]);

    const executable = findFile(workDir, [executableName]);
    if (!executable) throw new Error(`The ${sidecar.name} archive did not contain an executable`);
    copyFileSync(executable, sidecarPath);
    if (!isWindows) chmodSync(sidecarPath, 0o755);

    const licenseTarget = join(binariesDir, sidecar.licenseFile);
    const license = findFile(workDir, sidecar.licenseNames);
    if (license) {
      copyFileSync(license, licenseTarget);
    } else {
      writeFileSync(licenseTarget, sidecar.fallbackLicense(sidecar.version));
    }

    writeFileSync(versionMarker, markerValue);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

async function main() {
  const target = hostTargetTriple();
  mkdirSync(binariesDir, { recursive: true });
  for (const sidecar of sidecars) {
    await fetchSidecar(sidecar, target);
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
