#!/usr/bin/env node
import { buildBridge } from "@nsaga/build-tool";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

buildBridge({
  name: "bridge-nodejs",
  rootDir: __dirname,
  autoScanSrc: true,
});
