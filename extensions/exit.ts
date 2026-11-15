import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function (pi: ExtensionAPI): void {
  pi.registerCommand("exit", {
    description: "Gracefully exit Pi",
    handler: async (_args, ctx) => {
      ctx.shutdown();
    },
  });
}
