import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile, rename } from "node:fs/promises";

const filename = "Digital-Discipline-0.4.4-x64-setup.exe";
const expectedHash =
  "cee81d66fe23008bc8042de4e762210fc696898ef08575dee5f020298497f9d3";
const expectedSize = 5_981_800;
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
console.log("Windows installer verified (v0.4.4, SHA-256).");
