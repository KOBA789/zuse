async function main() {
  const canvas = document.createElement("canvas");
  canvas.id = "canvas";
  canvas.width = 1000;
  canvas.height = 1000;
  document.body.appendChild(canvas);
  await import("../rs/pkg");
}

main()
  .catch(console.error);
