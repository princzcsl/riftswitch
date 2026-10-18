// Génère latest.json (manifeste lu par le système de mise à jour) à partir du build signé.
// Usage : node scripts/release-manifest.mjs "Notes de version"
// À publier dans la release GitHub avec l'installateur et sa signature.
import fs from "fs";
import path from "path";

const version = JSON.parse(fs.readFileSync("package.json", "utf8")).version;
const setup = `RiftSwitch_${version}_x64-setup.exe`;
const dir = path.join("src-tauri", "target", "release", "bundle", "nsis");
const sigPath = path.join(dir, `${setup}.sig`);
if (!fs.existsSync(sigPath)) {
  console.error(`Signature introuvable : ${sigPath} (build lancé avec TAURI_SIGNING_PRIVATE_KEY ?)`);
  process.exit(1);
}
const manifest = {
  version,
  notes: process.argv[2] ?? "",
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: fs.readFileSync(sigPath, "utf8").trim(),
      url: `https://github.com/princzcsl/riftswitch/releases/download/v${version}/${setup}`,
    },
  },
};
const out = path.join(dir, "latest.json");
fs.writeFileSync(out, JSON.stringify(manifest, null, 2) + "\n");
console.log(`OK : ${out}`);
