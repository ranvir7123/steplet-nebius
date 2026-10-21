// The few Node functions the screen tests use, declared here because @types/node is not in the npm cache
// (a download would need the owner's yes, Z6). Playwright runs these tests in Node, where they exist.
declare module "node:url" {
  export function fileURLToPath(url: string | URL): string;
}
declare module "node:fs" {
  export function readFileSync(path: string | URL, encoding: "utf8"): string;
}
declare module "node:path" {
  export function basename(path: string): string;
  export function dirname(path: string): string;
}
declare const process: { env: Record<string, string | undefined> };
