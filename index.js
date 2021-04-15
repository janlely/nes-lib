const js = import("./node_modules/@janlely/nes-lib/nes_lib.js");
js.then(js => {
  js.greet("WebAssembly");
});
