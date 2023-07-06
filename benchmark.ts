import { createClient } from "https://esm.sh/v128/@redis/client@1.5.8";
import { generate } from "https://esm.sh/v128/randomstring@1.3.0";

const TOTAL_KEYS = 100_000;

const client = createClient();

await client.connect();

const keys = Array.from({ length: TOTAL_KEYS }, () => generate(100));

for (const key of keys) {
  await client.set(key, "value");
}

for (const key of keys) {
  await client.get(key);
}

for (const key of keys) {
  await client.del(key);
}

await client.disconnect();
