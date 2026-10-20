import { execFileSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const PANDOC_VERSION = "3.6.4";

const archiveByTarget = {
  "x86_64-unknown-linux-gnu": "linux-amd64.tar.gz",
  "aarch64-unknown-linux-gnu": "linux-arm64.tar.gz",
  "x86_64-apple-darwin": "x86_64-macOS.zip",
  "aarch64-apple-darwin": "arm64-macOS.zip",
  "x86_64-pc-windows-msvc": "windows-x86_64.zip",
};

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const binariesDir = join(projectRoot, "src-tauri", "binaries");
const versionMarker = join(binariesDir, "pandoc-version.txt");
const licenseTarget = join(binariesDir, "pandoc-COPYRIGHT.txt");

function hostTargetTriple() {
  if (process.env.TAURI_ENV_TARGET_TRIPLE) return process.env.TAURI_ENV_TARGET_TRIPLE;
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const hostLine = output.split("\n").find((line) => line.startsWith("host:"));
  if (!hostLine) throw new Error("Could not determine the Rust target triple");
  return hostLine.replace("host:", "").trim();
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

async function main() {
  const target = hostTargetTriple();
  const archiveSuffix = archiveByTarget[target];
  if (!archiveSuffix) throw new Error(`No pandoc build is configured for ${target}`);

  const isWindows = target.includes("windows");
  const sidecarPath = join(binariesDir, `pandoc-${target}${isWindows ? ".exe" : ""}`);
  const markerValue = `${PANDOC_VERSION} ${target}`;

  if (existsSync(sidecarPath) && existsSync(versionMarker) && readFileSync(versionMarker, "utf8").trim() === markerValue) {
    return;
  }

  const archiveName = `pandoc-${PANDOC_VERSION}-${archiveSuffix}`;
  const url = `https://github.com/jgm/pandoc/releases/download/${PANDOC_VERSION}/${archiveName}`;
  const workDir = join(tmpdir(), `typst-desktop-pandoc-${process.pid}`);
  const archivePath = join(workDir, archiveName);

  mkdirSync(workDir, { recursive: true });
  mkdirSync(binariesDir, { recursive: true });

  try {
    console.log(`Downloading pandoc ${PANDOC_VERSION} for ${target}`);
    await download(url, archivePath);
    execFileSync("tar", ["-xf", archivePath, "-C", workDir]);

    const executable = findFile(workDir, [isWindows ? "pandoc.exe" : "pandoc"]);
    if (!executable) throw new Error("The pandoc archive did not contain an executable");
    copyFileSync(executable, sidecarPath);
    if (!isWindows) chmodSync(sidecarPath, 0o755);

    const license = findFile(workDir, ["COPYRIGHT", "COPYRIGHT.txt", "COPYING.md", "COPYING.txt"]);
    if (license) {
      copyFileSync(license, licenseTarget);
    } else {
      writeFileSync(licenseTarget, `Pandoc ${PANDOC_VERSION} is licensed under the GNU GPL, version 2 or later.\nSource code: https://github.com/jgm/pandoc\n`);
    }

    writeFileSync(versionMarker, markerValue);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
