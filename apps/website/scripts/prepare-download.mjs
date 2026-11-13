import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile, rename } from "node:fs/promises";

const directory = new URL("../public/downloads/", import.meta.url);
const origin = "https://digital-discipline-windows.codecut-official.chatgpt.site";
const windows = JSON.parse(await readFile(new URL("../release.json", import.meta.url), "utf8"));
const android = JSON.parse(await readFile(new URL("../android-release.json", import.meta.url), "utf8"));

async function verifyDownload(platform, release, expectedName, minimumBytes) {
  const { filename, sha256, bytes: expectedSize, version } = release;
  if (
    !/^\d{1,4}\.\d{1,4}\.\d{1,4}$/.test(version) ||
    filename !== expectedName ||
    !/^[a-f0-9]{64}$/.test(sha256) ||
    !Number.isSafeInteger(expectedSize) ||
    expectedSize < minimumBytes ||
    expectedSize > 32 * 1024 * 1024
  ) throw new Error(`Invalid ${platform} release manifest`);

  const target = new URL(filename, directory);
  let bytes;
  let fetched = false;
  try {
    bytes = await readFile(target);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    const response = await fetch(`${origin}/downloads/${filename}`, {
      signal: AbortSignal.timeout(60_000),
      redirect: "error",
    });
    if (!response.ok) throw new Error(`${platform} download failed: HTTP ${response.status}`);
    bytes = Buffer.from(await response.arrayBuffer());
    fetched = true;
  }

  if (bytes.length !== expectedSize || createHash("sha256").update(bytes).digest("hex") !== sha256) {
    throw new Error(`${platform} download integrity check failed.`);
  }
  if (fetched) {
    await mkdir(directory, { recursive: true });
    const temporary = new URL(`${target.href}.tmp`);
    await writeFile(temporary, bytes);
    await rename(temporary, target);
  }
  console.log(`${platform} download verified (v${version}, SHA-256).`);
}

await verifyDownload("Windows", windows, `Digital-Discipline-${windows.version}-x64-setup.exe`, 100_000);
await verifyDownload("Android", android, `Digital-Discipline-Android-${android.version}-signed.apk`, 1);

// The Windows updater rejects unknown fields in this public manifest.
const { schema, version, companion_version, protocol, filename, bytes, sha256, signing, signer_thumbprint, notes } = windows;
const updater = { schema, version, companion_version, protocol, filename, bytes, sha256, signing, signer_thumbprint, notes };
await writeFile(new URL("latest.json", directory), JSON.stringify(updater, null, 2) + "\n");
