import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile, rename } from "node:fs/promises";

const manifest = await readFile(new URL("../release.json", import.meta.url), "utf8");
const { filename, sha256: expectedHash, bytes: expectedSize, version } =
  JSON.parse(manifest);
if (
  !/^\d{1,4}\.\d{1,4}\.\d{1,4}$/.test(version) ||
  filename !== "Digital-Discipline-" + version + "-x64-setup.exe" ||
  !/^[a-f0-9]{64}$/.test(expectedHash) ||
  !Number.isSafeInteger(expectedSize) ||
  expectedSize < 100_000 ||
  expectedSize > 32 * 1024 * 1024
) throw new Error("Invalid Windows release manifest");
const target = new URL(`../public/downloads/${filename}`, import.meta.url);

function verify(bytes) {
  if (
    bytes.length !== expectedSize ||
    createHash("sha256").update(bytes).digest("hex") !== expectedHash
  ) {
    throw new Error(
      "Installer integrity check failed. Refusing to build with an unverified download.",
    );
  }
}

let existing;
try {
  existing = await readFile(target);
} catch (error) {
  if (error.code !== "ENOENT") throw error;
}

if (existing) {
  verify(existing);
} else {
  const response = await fetch(
    `https://digital-discipline-windows.codecut-official.chatgpt.site/downloads/${filename}`,
    {
      signal: AbortSignal.timeout(60_000),
      redirect: "error",
    },
  );
  if (!response.ok)
    throw new Error(`Installer download failed: HTTP ${response.status}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  verify(bytes);
  await mkdir(new URL("../public/downloads/", import.meta.url), {
    recursive: true,
  });
  const temporary = new URL(`${target.href}.tmp`);
  await writeFile(temporary, bytes);
  await rename(temporary, target);
}
await writeFile(new URL("../public/downloads/latest.json", import.meta.url), manifest);
console.log("Windows installer verified (v" + version + ", SHA-256).");
