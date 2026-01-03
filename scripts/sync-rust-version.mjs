import fs from "node:fs";

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, "utf8"));
}

function replaceCargoVersion(toml, version) {
  const lines = toml.split(/\r?\n/);
  let inPackage = false;
  let changed = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    if (/^\s*\[package\]\s*$/.test(line)) {
      inPackage = true;
      continue;
    }
    if (/^\s*\[.*\]\s*$/.test(line) && !/^\s*\[package\]\s*$/.test(line)) {
      if (inPackage) break;
      continue;
    }

    if (inPackage && /^\s*version\s*=/.test(line)) {
      lines[i] = `version = "${version}"`;
      changed = true;
      break;
    }
  }

  if (!changed) {
    throw new Error("Failed to update Cargo.toml: version field not found under [package]");
  }

  return lines.join("\n");
}

const pkg = readJson("package.json");
const version = pkg.version;

if (!version || typeof version !== "string") {
  throw new Error("package.json version missing");
}

const cargoPath = "app/Cargo.toml";
const cargoToml = fs.readFileSync(cargoPath, "utf8");
const updated = replaceCargoVersion(cargoToml, version);
fs.writeFileSync(cargoPath, updated);
console.log(`Synced Rust Cargo version -> ${version}`);
