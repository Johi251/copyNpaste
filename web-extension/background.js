/** Clipboard-Zugriff gehört in ein Dokument, nicht in einen Service Worker. */
const CAPACITY = 25;
let pending = Promise.resolve();
function updateMagazine(change) {
  pending = pending.catch(() => {}).then(async () => {
    const { magazine = [] } = await chrome.storage.local.get("magazine");
    if (!Array.isArray(magazine)) throw new Error("Magazin ist beschädigt; Bestand bleibt erhalten");
    await chrome.storage.local.set({ magazine: change(magazine).slice(0, CAPACITY) });
  });
  return pending;
}
function storeText(text) {
  if (typeof text !== "string" || !text.trim() || text.length > 100_000) return Promise.resolve();
  return updateMagazine(magazine => magazine[0]?.text === text ? magazine :
    [{ id: crypto.randomUUID(), text, at: Date.now() }, ...magazine]);
}
chrome.runtime.onMessage.addListener((message, _sender, respond) => {
  if (message?.target !== "magazine") return;
  let result;
  if (message.type === "capture") result = storeText(message.text);
  else if (message.type === "consume" && typeof message.id === "string")
    result = updateMagazine(magazine => magazine.filter(item => item.id !== message.id));
  else if (message.type === "clear") result = updateMagazine(() => []);
  else return;
  result.then(() => respond({ ok: true }), e => respond({ error: String(e) }));
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
