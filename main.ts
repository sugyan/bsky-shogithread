import { shogi_thread } from "./pkg/bsky_shogithread.js";

Deno.cron("shogi thread", { minute: { every: 1 } }, async () => {
  const kv = await Deno.openKv();
  const input = {
    config: (await kv.get(["config"])).value,
    identifier: Deno.env.get("BSKY_IDENTIFIER"),
    password: Deno.env.get("BSKY_PASSWORD"),
  };
  try {
    const result = await shogi_thread(JSON.stringify(input));
    kv.set(["config"], JSON.parse(result));
  } catch (e) {
    console.error(e);
  }
});
