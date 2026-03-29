import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");
const debugApiBase = process.env.RING_DEBUG_API_BASE ?? "http://127.0.0.1:9528";
const outputPath =
  process.env.RING_HARNESS_OUTPUT ??
  path.join(repoRoot, "artifacts", "host-tauri", "harness-smoke-bundle.json");
const dt = Number(process.env.RING_HARNESS_DT ?? 1 / 60);
const maxSteps = Number(process.env.RING_HARNESS_MAX_STEPS ?? 600);
const startLabel = process.env.RING_HARNESS_LABEL ?? "";

async function call(command, args = {}) {
  const resp = await fetch(`${debugApiBase}/api/${command}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args),
  });

  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`${command} failed: ${resp.status} ${text}`);
  }

  const text = await resp.text();
  return text ? JSON.parse(text) : null;
}

async function main() {
  const session = await call("frontend_connected", { clientLabel: "harness-smoke" });
  const clientToken = session.client_token;
  const config = await call("get_config");
  const scriptPath = process.env.RING_HARNESS_SCRIPT ?? config.start_script_path;

  if (startLabel) {
    await call("init_game_at_label", {
      clientToken,
      scriptPath,
      label: startLabel,
    });
  } else {
    await call("init_game", {
      clientToken,
      scriptPath,
    });
  }

  const bundle = await call("debug_run_until", {
    clientToken,
    dt,
    maxSteps,
    stopOnWait: true,
    stopOnScriptFinished: true,
  });

  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, `${JSON.stringify(bundle, null, 2)}\n`, "utf8");

  const reason = bundle?.metadata?.stop_reason ?? "unknown";
  const steps = bundle?.metadata?.steps_run ?? 0;
  console.log(`Harness smoke complete: stop_reason=${reason}, steps=${steps}`);
  console.log(`Bundle written to: ${outputPath}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
