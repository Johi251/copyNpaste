document.addEventListener("copy", event => {
  const field = document.activeElement;
  if (field instanceof HTMLInputElement && (field.type === "password" || /password/i.test(field.autocomplete))) return;
  const text = event.clipboardData?.getData("text/plain") ||
    (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement
      ? field.value.slice(field.selectionStart ?? 0, field.selectionEnd ?? 0)
      : window.getSelection()?.toString());
  if (text && text.trim()) chrome.runtime.sendMessage({ target: "magazine", type: "capture", text }).catch(() => {});
});
