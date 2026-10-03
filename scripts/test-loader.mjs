import { registerHooks } from "node:module";
import { statSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const LIB = fileURLToPath(new URL("../src/lib/", import.meta.url));

const isFile = (p) => statSync(p, { throwIfNoEntry: false })?.isFile() ?? false;

// Vite resolves `$lib` and extensionless/directory imports; node --test does not.
function locate(base) {
  return [base, `${base}.ts`, path.join(base, "index.ts")].find(isFile);
}

registerHooks({
  resolve(specifier, context, next) {
    if (specifier.startsWith("$lib/")) {
      const hit = locate(path.join(LIB, specifier.slice(5)));
      if (hit) return next(pathToFileURL(hit).href, context);
    }
    try {
      return next(specifier, context);
    } catch (err) {
      if (!specifier.startsWith(".") || !context.parentURL) throw err;
      const hit = locate(fileURLToPath(new URL(specifier, context.parentURL)));
      if (!hit) throw err;
      return next(pathToFileURL(hit).href, context);
    }
  },
});
