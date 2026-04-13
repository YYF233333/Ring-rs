/**
 * Ring Engine Debug MCP Server
 *
 * Thin MCP wrapper over the debug HTTP REST API embedded in host-dioxus.
 * Runs as a stdio MCP server — Claude Code connects via mcpServers config.
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const BASE_URL = `http://127.0.0.1:${process.env.RING_DEBUG_PORT || 9876}`;

// ── HTTP helpers ─────────────────────────────────────────────────────────────

async function httpGet(path) {
  const res = await fetch(`${BASE_URL}${path}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`HTTP ${res.status}: ${text}`);
  }
  return res.json();
}

async function httpPost(path, body = undefined) {
  const opts = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  };
  if (body !== undefined) {
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(`${BASE_URL}${path}`, opts);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`HTTP ${res.status}: ${text}`);
  }
  return res.json();
}

function jsonText(obj) {
  return JSON.stringify(obj, null, 2);
}

// ── MCP Server ───────────────────────────────────────────────────────────────

const server = new McpServer({
  name: "ring-debug",
  version: "0.1.0",
});

// ── Tools: State Queries ─────────────────────────────────────────────────────

server.tool("ping", "Check if the game is running and get basic status", {}, async () => {
  try {
    const data = await httpGet("/api/ping");
    return { content: [{ type: "text", text: jsonText(data) }] };
  } catch (e) {
    return {
      content: [{ type: "text", text: `Game not reachable: ${e.message}` }],
      isError: true,
    };
  }
});

server.tool(
  "get_game_state",
  "Get full game state: render state, waiting status, playback mode, screen",
  {},
  async () => {
    const data = await httpGet("/api/state");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "get_dialogue",
  "Get current dialogue state: speaker, text, typewriter progress, NVL entries",
  {},
  async () => {
    const data = await httpGet("/api/state/dialogue");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "get_scene",
  "Get visual scene state: background, characters, transitions, effects",
  {},
  async () => {
    const data = await httpGet("/api/state/scene");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "get_choices",
  "Get current choice options (only meaningful when waiting for choice)",
  {},
  async () => {
    const data = await httpGet("/api/state/choices");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool("get_audio", "Get audio state: BGM, SFX", {}, async () => {
  const data = await httpGet("/api/state/audio");
  return { content: [{ type: "text", text: jsonText(data) }] };
});

// ── Tools: Actions ───────────────────────────────────────────────────────────

server.tool("click", "Click to advance dialogue (equivalent to mouse click / Enter)", {}, async () => {
  const data = await httpPost("/api/click");
  return { content: [{ type: "text", text: jsonText(data) }] };
});

server.tool(
  "choose",
  "Select a choice option by index (0-based)",
  { index: z.number().int().min(0).describe("Choice index (0-based)") },
  async ({ index }) => {
    const data = await httpPost("/api/choose", { index });
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "advance",
  "Click multiple times until a meaningful wait state (choice, signal, etc.) or max_clicks reached",
  {
    max_clicks: z
      .number()
      .int()
      .min(1)
      .max(100)
      .default(10)
      .describe("Maximum number of clicks (default 10, max 100)"),
  },
  async ({ max_clicks }) => {
    const data = await httpPost("/api/advance", { max_clicks });
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "navigate",
  "Switch host screen (title, ingame, save, load, settings, history)",
  {
    screen: z
      .string()
      .describe("Target screen: title, ingame, ingamemenu, save, load, settings, history"),
  },
  async ({ screen }) => {
    const data = await httpPost("/api/navigate", { screen });
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "start_game",
  "Start a new game from a script file, optionally at a specific label",
  {
    script: z.string().optional().describe("Script path (defaults to config start_script_path)"),
    label: z.string().optional().describe("Jump to this label after loading"),
  },
  async ({ script, label }) => {
    const body = {};
    if (script) body.script = script;
    if (label) body.label = label;
    const data = await httpPost("/api/start_game", body);
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "set_playback_mode",
  "Set playback mode: normal, auto, or skip",
  {
    mode: z.string().describe("Playback mode: normal, auto, skip"),
  },
  async ({ mode }) => {
    const data = await httpPost("/api/playback_mode", { mode });
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

// ── Tools: Screenshot ────────────────────────────────────────────────────────

server.tool(
  "screenshot",
  "Capture a screenshot of the current game view (returns base64 PNG image)",
  {},
  async () => {
    const data = await httpGet("/api/screenshot");
    if (data.data_base64) {
      return {
        content: [
          {
            type: "image",
            data: data.data_base64,
            mimeType: "image/png",
          },
        ],
      };
    }
    return {
      content: [{ type: "text", text: jsonText(data) }],
      isError: true,
    };
  },
);

// ── Tools: Diagnostics ───────────────────────────────────────────────────────

server.tool(
  "check_transitions",
  "Check if any transitions or animations are currently in progress",
  {},
  async () => {
    const data = await httpGet("/api/diag/transitions");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

server.tool(
  "check_typewriter",
  "Check typewriter animation progress (visible chars, total, completion)",
  {},
  async () => {
    const data = await httpGet("/api/diag/typewriter");
    return { content: [{ type: "text", text: jsonText(data) }] };
  },
);

// ── Start ────────────────────────────────────────────────────────────────────

const transport = new StdioServerTransport();
await server.connect(transport);
