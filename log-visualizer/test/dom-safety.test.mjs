import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const fixture = JSON.parse(await readFile(new URL("test/fixtures/hostile-dom-inputs.json", root)));
const pages = await Promise.all(
  ["public/index.html", "public/mesh.html", "public/wiring.html"].map(async (path) => ({
    path,
    source: await readFile(new URL(path, root), "utf8"),
  })),
);

assert.equal(fixture.length, 6, "fixture must cover each hostile input category");
assert.match(fixture[0], /[<>]/, "fixture covers angle brackets");
assert.match(fixture[1], /['\"]/, "fixture covers quotes");
assert.match(fixture[2], /&/, "fixture covers ampersands");
assert.match(fixture[3], /onerror/i, "fixture covers event-handler text");
assert.match(fixture[4], /<\/script/i, "fixture covers script-closing text");
assert.match(fixture[5], /12D3KooW.*<script/i, "fixture covers malicious peer IDs");

const index = pages.find(({ path }) => path.endsWith("index.html")).source;
assert.match(index, /tooltip\.textContent\s*=\s*d\.name/);
assert.match(index, /level\.textContent\s*=\s*log\.levelTag/);
assert.match(index, /document\.createTextNode\(`: \$\{log\.msg/);
assert.doesNotMatch(index, /innerHTML\s*=\s*d\.name/);
assert.doesNotMatch(index, /innerHTML\s*=\s*`<span[^`]*\$\{log\./);
assert.doesNotMatch(index, /innerHTML/);

const mesh = pages.find(({ path }) => path.endsWith("mesh.html")).source;
assert.match(mesh, /peer\.textContent\s*=\s*peerId/);
assert.match(mesh, /endpoints\.textContent\s*=\s*`\$\{src\} ↔ \$\{tgt\}`/);
assert.doesNotMatch(mesh, /\.html\(`[^`]*\$\{(?:d\.name|peerId|src|tgt)/s);
assert.doesNotMatch(mesh, /\.html\(/);

const wiring = pages.find(({ path }) => path.endsWith("wiring.html")).source;
assert.match(wiring, /\$tooltip\.replaceChildren/);
assert.match(wiring, /\$detail\.replaceChildren/);
assert.match(wiring, /link\.addEventListener\("click"/);
assert.doesNotMatch(wiring, /onclick="jumpToNode/);
assert.doesNotMatch(wiring, /\$tooltip\.innerHTML\s*=\s*`/);
assert.doesNotMatch(wiring, /innerHTML/);

console.log("[OK] hostile DOM-input safety checks passed");
