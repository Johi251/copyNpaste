chrome.runtime.onMessage.addListener((message, _sender, respond) => {
  if (message?.target !== "clipboard" || message.type !== "read") return;
  (async () => {
    try { return await navigator.clipboard.readText(); }
    catch {
      const field = document.getElementById("clipboard");
      field.value = ""; field.focus();
      if (!document.execCommand("paste")) throw new Error("Clipboard-Zugriff verweigert");
      const text = field.value; field.value = ""; return text;
    }
  })().then(text => respond({ text }), e => respond({ error: String(e) }));
  return true;
});
