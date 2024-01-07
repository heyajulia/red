import { createClient } from "https://esm.sh/v128/@redis/client@1.5.8";
import { generate as randomString } from "https://esm.sh/v128/randomstring@1.3.0";
import { sample as choose } from "https://esm.sh/v128/lodash-es@4.17.21";

const lsof = new Deno.Command("lsof", {
  args: ["-i", "tcp:6379"],
  stdout: "piped",
});

const output = await lsof.output();

console.log("Benchmarking the following Redis instance:");
console.log(new TextDecoder().decode(output.stdout));

const SECONDS = 5;

let totalCount = 0;
const operations = ["GET", "SET", "DEL", "PING"];

const client = createClient();

await client.connect();

const performOperation = () => {
  const operation = choose(operations);
  const key = randomString();
  const value = randomString();

  switch (operation) {
    case "GET":
      client.get(key).then(() => totalCount++);
      break;
    case "SET":
      client.set(key, value).then(() => totalCount++);
      break;
    case "DEL":
      client.del(key).then(() => totalCount++);
      break;
    case "PING":
      client.ping().then(() => totalCount++);
      break;
  }
};

const interval = setInterval(performOperation, 1); // Adjust the interval as needed

setTimeout(() => {
  clearInterval(interval);
  const averageRPS = totalCount / SECONDS;
  console.log(`Average RPS: ${averageRPS}`);
  // FIXME: client.quit() causes an error with Red :(
  Deno.exit(0);
}, SECONDS * 1000);
