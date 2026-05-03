#!/usr/bin/env node
// Ensures the platform native package is present after npm install.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const packageRoot = path.join(__dirname, "..");
const require = createRequire(import.meta.url);

const PLATFORM_PACKAGE_BY_TARGET = {
  "x86_64-unknown-linux-musl": "@growthcircle/growcli-linux-x64",
  "aarch64-unknown-linux-musl": "@growthcircle/growcli-linux-arm64",
  "x86_64-apple-darwin": "@growthcircle/growcli-darwin-x64",
  "aarch64-apple-darwin": "@growthcircle/growcli-darwin-arm64",
  "x86_64-pc-windows-msvc": "@growthcircle/growcli-win32-x64",
  "aarch64-pc-windows-msvc": "@growthcircle/growcli-win32-arm64",
};

const packageJson = readPackageJson();
const targetTriple = detectTargetTriple(process.platform, process.arch);
const platformPackage = targetTriple
  ? PLATFORM_PACKAGE_BY_TARGET[targetTriple]
  : null;

if (!platformPackage) {
  process.exit(0);
}

if (hasPackage(platformPackage)) {
  process.exit(0);
}

const npm = npmCommand();
const version = packageJson.version;
const result = spawnSync(
  npm.command,
  [
    ...npm.args,
    "install",
    "--no-save",
    "--package-lock=false",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--include=optional",
    "--prefix",
    packageRoot,
    `${platformPackage}@${version}`,
  ],
  { stdio: "inherit", env: process.env },
);

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);

function detectTargetTriple(platform, arch) {
  switch (platform) {
    case "linux":
    case "android":
      return arch === "x64"
        ? "x86_64-unknown-linux-musl"
        : arch === "arm64"
          ? "aarch64-unknown-linux-musl"
          : null;
    case "darwin":
      return arch === "x64"
        ? "x86_64-apple-darwin"
        : arch === "arm64"
          ? "aarch64-apple-darwin"
          : null;
    case "win32":
      return arch === "x64"
        ? "x86_64-pc-windows-msvc"
        : arch === "arm64"
          ? "aarch64-pc-windows-msvc"
          : null;
    default:
      return null;
  }
}

function hasPackage(packageName) {
  try {
    require.resolve(`${packageName}/package.json`);
    return true;
  } catch {
    return false;
  }
}

function npmCommand() {
  const npmExecPath = process.env.npm_execpath;
  if (npmExecPath && existsSync(npmExecPath)) {
    return { command: process.execPath, args: [npmExecPath] };
  }

  return {
    command: process.platform === "win32" ? "npm.cmd" : "npm",
    args: [],
  };
}

function readPackageJson() {
  return JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8"));
}
