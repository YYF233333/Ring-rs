import { spawn } from "node:child_process";

const command = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
const child = spawn(command, ["tauri", "dev"], {
  cwd: new URL("..", import.meta.url),
  stdio: "inherit",
  shell: false,
  env: {
    ...process.env,
    RING_HEADLESS: "1",
  },
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});
