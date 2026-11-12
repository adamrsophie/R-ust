const status = document.getElementById("copy-status");
let statusTimer;
function announce(message) {
  clearTimeout(statusTimer);
  status.textContent = message;
  statusTimer = setTimeout(() => { status.textContent = ""; }, 5000);
}

for (const button of document.querySelectorAll("[data-copy]")) {
  button.hidden = false;
  button.addEventListener("click", async () => {
    const code = document.getElementById(button.dataset.copy);
    try {
      await navigator.clipboard.writeText(code.textContent);
      announce("Copied to clipboard.");
    } catch {
      const selection = window.getSelection();
      const range = document.createRange();
      range.selectNodeContents(code);
      selection.removeAllRanges();
      selection.addRange(range);
      announce("Text selected. Use your browser’s Copy command.");
    }
  });
}

try {
  const response = await fetch(new URL("./examples.json", import.meta.url));
  if (!response.ok) throw new Error("Examples unavailable");
  const examples = await response.json();
  const choices = document.querySelector(".example-choices");
  const buttons = [...choices.querySelectorAll("button")];
  for (const button of buttons) {
    const example = examples.find(item => item.id === button.dataset.example);
    if (!example) throw new Error("Missing example");
    button.addEventListener("click", () => {
      document.getElementById("example-code").textContent = example.code;
      document.getElementById("example-output").textContent = example.output;
      for (const choice of buttons) choice.setAttribute("aria-pressed", String(choice === button));
    });
  }
  choices.hidden = false;
} catch {
  // Keep the initial captured example readable if the optional data request fails.
}
