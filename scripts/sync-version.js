import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "..");

// Read the version npm just wrote to package.json
const pkgPath = join(rootDir, "package.json");
const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
const newVersion = pkg.version;

console.log(
  `Syncing version ${newVersion} into tauri.conf.json and Cargo.toml...`
);

// --- tauri.conf.json ---
const tauriConfPath = join(rootDir, "src-tauri", "tauri.conf.json");
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf-8"));
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

// --- Cargo.toml ---
const cargoTomlPath = join(rootDir, "src-tauri", "Cargo.toml");
let cargoToml = readFileSync(cargoTomlPath, "utf-8");
cargoToml = cargoToml.replace(
  /^version\s*=\s*".*"$/m,
  `version = "${newVersion}"`
);
writeFileSync(cargoTomlPath, cargoToml);

console.log("Done.");
