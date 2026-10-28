/**
 * copyNpaste Browser-Add-on — Service Worker.
 * Hält den Befehl "push-clipboard" bereit: schiebt die aktuelle Zwischenablage
 * ins Magazin, ohne das Popup öffnen zu müssen (funktioniert, wenn der Browser
 * fokussiert ist; sonst ist der Popup-Weg der zuverlässige Pfad).
 */

const CAPACITY = 25;

chrome.commands.onCommand.addListener(async (command) => {
  if (command !== "push-clipboard") return;
  try {
    const text = (await navigator.clipboard.readText() ?? "").trim();
    if (!text || text.length > 100_000) return;
    const { magazine = [] } = await chrome.storage.local.get("magazine");
    if (magazine[0]?.text === text) return;
    magazine.unshift({ id: crypto.randomUUID(), text, at: Date.now() });
    await chrome.storage.local.set({ magazine: magazine.slice(0, CAPACITY) });
    await chrome.action.setBadgeText({ text: String(Math.min(magazine.length, 99)) });
    await chrome.action.setBadgeBackgroundColor({ color: "#0D9488" });
    setTimeout(() => chrome.action.setBadgeText({ text: "" }), 1200);
  } catch {
    // Clipboard-Lesen ohne Nutzer-Geste nicht erlaubt — Popup-Weg nutzen.
  }
});
