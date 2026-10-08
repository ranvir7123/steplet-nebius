// Milestone 0 stand-in gate: a Claude Code permission-prompt tool whose behaviour is chosen by GATE_MODE.
// Modes: allow, deny, allowlist, crash, hang, garbage, wrongshape. Logs every request it receives.
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { appendFileSync } from "node:fs";

const mode = process.env.GATE_MODE || "allow";
const log = process.env.GATE_LOG || "/home/ranvir/m0/logs/gate.log";
const say = (o) => ({ content: [{ type: "text", text: typeof o === "string" ? o : JSON.stringify(o) }] });
const server = new McpServer({ name: "gate", version: "0.0.1" });

// Everything optional and unknown keys kept, so the log shows exactly what the agent sends.
const request = z.looseObject({
  tool_name: z.string().optional(),
  input: z.record(z.string(), z.any()).optional(),
  tool_use_id: z.string().optional(),
});

server.registerTool("decide", { description: "Milestone 0 permission gate", inputSchema: request }, async (args) => {
  appendFileSync(log, new Date().toISOString() + " " + mode + " " + JSON.stringify(args) + "\n");
  if (mode === "crash") process.exit(1);
  if (mode === "hang") return new Promise(() => {});
  if (mode === "garbage") return say("this is not json");
  if (mode === "wrongshape") return say({ behavior: "maybe" });
  if (mode === "deny") return say({ behavior: "deny", message: "m0 test: denied", interrupt: false });
  if (mode === "allowlist") {
    const cmd = (args.input && args.input.command) || "";
    const ok = args.tool_name === "Bash" && /^(git (checkout|rev-parse|status)|python3? -m unittest)/.test(cmd);
    return say(ok ? { behavior: "allow", updatedInput: args.input }
                  : { behavior: "deny", message: "m0 test: not on the list", interrupt: false });
  }
  return say({ behavior: "allow", updatedInput: args.input });
});

await server.connect(new StdioServerTransport());
