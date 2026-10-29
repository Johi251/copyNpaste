/** Clipboard-Zugriff gehört in ein Dokument, nicht in einen Service Worker. */
const CAPACITY = 25;
let pending = Promise.resolve();
function storeText(text) {
  pending = pending.catch(() => {}).then(async () => {
    if (typeof text !== "string" || !text.trim() || text.length > 100_000) return;
    const { magazine = [] } = await chrome.storage.local.get("magazine");
    if (magazine[0]?.text === text) return;
    magazine.unshift({ id: crypto.randomUUID(), text, at: Date.now() });
    await chrome.storage.local.set({ magazine: magazine.slice(0, CAPACITY) });
  });
  return pending;
}
chrome.runtime.onMessage.addListener((message, _sender, respond) => {
  if (message?.target !== "magazine" || message.type !== "capture") return;
  storeText(message.text).then(() => respond({ ok: true }), e => respond({ error: String(e) }));
  return true;
});
chrome.commands.onCommand.addListener(async command => {
  if (command !== "push-clipboard") return;
  try {
    let text;
    if (navigator.clipboard) { text = await navigator.clipboard.readText(); }
    else {
      if (!(await chrome.runtime.getContexts({ contextTypes: ["OFFSCREEN_DOCUMENT"] })).length) {
        await chrome.offscreen.createDocument({ url: "offscreen.html", reasons: ["CLIPBOARD"], justification: "Zwischenablage nach ausdrücklichem Shortcut aufnehmen" });
      }
      const result = await chrome.runtime.sendMessage({ target: "clipboard", type: "read" });
      if (result?.error) throw new Error(result.error);
      text = result?.text;
    }
    await storeText(text);
    await chrome.action.setBadgeText({ text: "" });
  } catch (e) {
    await chrome.action.setBadgeText({ text: "!" });
    await chrome.action.setTitle({ title: "Zwischenablage konnte nicht gelesen werden. Bitte Popup öffnen. " + String(e) });
  }
});
