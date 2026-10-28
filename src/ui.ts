/** UI-Bausteine: Icons (Lucide-Pfade), Karten-Rendering, Spalten, Leerezustände. */
import type { ItemLite } from "./types";

// ─── Icons (Lucide, inline — keine Laufzeit-Abhängigkeit) ───────────────
const svg = (paths: string, extra = "") =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" ${extra}>${paths}</svg>`;

export const icons = {
  settings: svg('<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>'),
  minimize: svg('<path d="M5 12h14"/>'),
  close: svg('<path d="M18 6 6 18"/><path d="m6 6 12 12"/>'),
  search: svg('<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>'),
  trash: svg('<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>'),
  text: svg('<path d="M17 6.1H3"/><path d="M21 12.1H3"/><path d="M15.1 18H3"/>'),
  media: svg('<rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>'),
  file: svg('<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>'),
  clipboard: svg('<rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>'),
  arrowDownToLine: svg('<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>'),
  pin: svg('<line x1="12" x2="12" y1="17" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/>'),
  link: svg('<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>'),
};

/** Marken-Logo: gestapelte Magazin-Balken in Türkis-Jade (#0D9488). */
export const logoSvg = `<svg viewBox="0 0 24 24" fill="none">
  <rect x="3" y="14.5" width="18" height="4" rx="1.4" fill="#134e4a"/>
  <rect x="3" y="8.75" width="18" height="4" rx="1.4" fill="#0f766e"/>
  <rect x="3" y="3" width="13.5" height="4" rx="1.4" fill="#14b8a6"/>
  <path d="M20.2 3.6 22 5.4l-2.9 2.9-1.8-1.8z" fill="#5eead4"/>
</svg>`;

// ─── Leerezustände ───────────────────────────────────────────────────────
function emptyState(kind: "text" | "media"): HTMLDivElement {
  const el = document.createElement("div");
  el.className = "flex h-full flex-col items-center justify-center gap-3 py-16 text-neutral-700";
  const icon = kind === "text" ? icons.clipboard : icons.media;
  el.innerHTML = `
    <div class="grid size-11 place-items-center rounded-xl border border-neutral-800/80 bg-neutral-900/40 [&_svg]:size-5">${icon}</div>
    <div class="text-center">
      <div class="text-[12px] text-neutral-500">${
        kind === "text" ? "Noch keine Texte im Magazin" : "Noch keine Bilder oder Dateien im Magazin"
      }</div>
      <div class="mt-1 text-[11px] text-neutral-700">Strg+C irgendwo im System — copyNpaste legt automatisch nach</div>
    </div>`;
  return el;
}

// ─── Haken-Animation unter dem Link-Anfang ──────────────────────────────
/** IDs, deren Erfolgs-Haken bereits einmal animiert wurde (kein Re-Play bei Re-Render). */
const animatedChecks = new Set<string>();

function urlBody(item: ItemLite): HTMLDivElement {
  const wrap = document.createElement("div");
  wrap.className = "relative mt-1";

  const link = document.createElement("span");
  link.className = "block break-all font-mono text-[12.5px] leading-relaxed text-teal-300";
  link.textContent = item.textPreview ?? item.preview;

  // Haken + Badge direkt unter dem Link-Anfang (Position 0 der Zeile)
  const under = document.createElement("span");
  under.className = "mt-1 flex items-center gap-1.5";

  if (item.bridge === "sent") {
    const first = !animatedChecks.has(item.id);
    if (first) animatedChecks.add(item.id);
    under.innerHTML = `
      <span class="${first ? "check-anim" : ""} inline-flex">
        <svg class="check-under-link" width="15" height="13" viewBox="0 0 15 13"><path d="M1.5 7.5 5 11 13.5 2"/></svg>
      </span>
      <span class="check-pop ${first ? "" : "!opacity-100"} inline-flex items-center gap-1 rounded border border-teal-900/60 bg-teal-950/40 px-1.5 py-px text-[10px] text-teal-400">
        ${item.isGithub ? "GitHub" : "URL"} → Today's Hunt
      </span>`;
  } else if (item.bridge === "pending") {
    under.innerHTML = `<span class="inline-flex items-center gap-1.5 text-[10px] text-teal-700">
      <span class="size-1.5 animate-pulse rounded-full bg-teal-600"></span> Bridge sendet …
    </span>`;
  } else if (item.bridge === "endpoint-offline") {
    under.innerHTML = `<span class="inline-flex items-center gap-1 rounded border border-amber-900/50 bg-amber-950/30 px-1.5 py-px text-[10px] text-amber-500" title="Bookmarks Dynamic Manager nicht erreichbar — URL bleibt im Magazin">
      Bridge offline — im Magazin behalten
    </span>`;
  }
  wrap.append(link, under);
  return wrap;
}

// ─── Karten ──────────────────────────────────────────────────────────────
export interface CardActions {
  onPaste: (id: string) => void;
  onDelete: (id: string) => void;
  onPin: (id: string) => void;
}

/** ID des Elements, das der nächste Alt+V einfügen würde (jüngstes ungepinntes). */
export function nextPasteId(items: ItemLite[]): string | undefined {
  return items.find((i) => !i.pinned)?.id;
}

export function makeCard(item: ItemLite, actions: CardActions, nextId?: string): HTMLElement {
  const isNext = item.id === nextId;
  const card = document.createElement("div");
  card.className = `cnp-card card-enter${isNext ? " is-next" : ""}${item.pinned ? " is-pinned" : ""}`;
  card.tabIndex = 0;
  card.dataset.id = item.id;
  card.setAttribute("role", "button");
  card.setAttribute(
    "aria-label",
    `Magazin-Platz ${item.position}: ${item.kind}${item.pinned ? ", gepinnt" : ""} — Enter zum Einfügen, Entf zum Entfernen`
  );

  // Kopfzeile: Position, ggf. „Nächster"-Marke, Zeitstempel, Aktionen
  const head = document.createElement("div");
  head.className = "flex items-center gap-2";
  const pos = document.createElement("span");
  pos.className =
    "grid size-[18px] shrink-0 place-items-center rounded bg-neutral-800/90 font-mono text-[10px] tabular-nums text-neutral-500";
  pos.textContent = String(item.position);
  head.append(pos);

  if (item.pinned) {
    const pin = document.createElement("span");
    pin.className =
      "inline-flex items-center gap-1 rounded border border-teal-900/60 bg-teal-950/40 px-1.5 py-px text-[9.5px] font-medium uppercase tracking-wider text-teal-500";
    pin.innerHTML = `<span class="[&_svg]:size-2.5">${icons.pin}</span>Gepinnt`;
    pin.title = "Gepinnt: bleibt im Magazin und wird von Alt+V übersprungen";
    head.append(pin);
  }

  if (isNext) {
    const next = document.createElement("span");
    next.className =
      "rounded border border-teal-800/70 bg-teal-950/50 px-1.5 py-px text-[9.5px] font-medium uppercase tracking-wider text-teal-400";
    next.textContent = "Nächster Paste";
    head.append(next);
  }

  const time = document.createElement("span");
  time.className = "text-[10.5px] tabular-nums text-neutral-600";
  time.textContent = item.capturedAt;
  head.append(time);

  const spacer = document.createElement("div");
  spacer.className = "flex-1";
  head.append(spacer);

  const actionsEl = document.createElement("div");
  actionsEl.className = "card-actions flex items-center gap-1";
  const pasteBtn = document.createElement("button");
  pasteBtn.className = "icon-btn !size-6 hover:!text-teal-400";
  pasteBtn.title = "Jetzt einfügen (aus dem Magazin entnehmen)";
  pasteBtn.innerHTML = icons.arrowDownToLine;
  pasteBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    actions.onPaste(item.id);
  });
  const pinBtn = document.createElement("button");
  pinBtn.className = `icon-btn !size-6 ${item.pinned ? "!text-teal-500" : "hover:!text-teal-400"}`;
  pinBtn.title = item.pinned
    ? "Pin lösen (wieder automatisch im Magazin-Rotation)"
    : "Anpinnen (bleibt im Magazin, Alt+V überspringt es)";
  pinBtn.innerHTML = icons.pin;
  pinBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    actions.onPin(item.id);
  });
  const delBtn = document.createElement("button");
  delBtn.className = "icon-btn !size-6 hover:!text-red-400";
  delBtn.title = "Aus dem Magazin entfernen";
  delBtn.innerHTML = icons.trash;
  delBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    actions.onDelete(item.id);
  });
  actionsEl.append(pasteBtn, pinBtn, delBtn);
  head.append(actionsEl);
  card.append(head);

  // Inhalt nach Art
  if (item.kind === "text") {
    if (item.isUrl) {
      card.append(urlBody(item));
    } else {
      const body = document.createElement("div");
      body.className =
        "mt-1.5 line-clamp-4 whitespace-pre-wrap break-all font-mono text-[12px] leading-relaxed text-neutral-300";
      body.textContent = item.textPreview ?? item.preview;
      card.append(body);
      const meta = document.createElement("div");
      meta.className = "mt-1.5 text-[10px] text-neutral-600";
      meta.textContent = item.meta;
      card.append(meta);
    }
  } else if (item.kind === "image") {
    const body = document.createElement("div");
    body.className = "mt-1.5 flex items-start gap-3";
    if (item.thumb) {
      const img = document.createElement("img");
      img.src = item.thumb;
      img.alt = `Bildvorschau ${item.meta}`;
      img.className = "max-h-40 rounded-md border border-neutral-800 object-contain";
      img.draggable = false;
      body.append(img);
    }
    const meta = document.createElement("div");
    meta.className = "text-[10.5px] tabular-nums text-neutral-500";
    meta.textContent = item.meta;
    body.append(meta);
    card.append(body);
  } else {
    const body = document.createElement("div");
    body.className = "mt-1.5 space-y-1";
    const files = (item.textPreview ?? item.preview).split("\n").filter(Boolean);
    for (const name of files.slice(0, 3)) {
      const row = document.createElement("div");
      row.className = "flex items-center gap-2 text-[12px] text-neutral-300";
      row.innerHTML = `<span class="shrink-0 text-neutral-600 [&_svg]:size-3.5">${icons.file}</span>`;
      const nm = document.createElement("span");
      nm.className = "truncate font-mono text-[11.5px]";
      nm.textContent = name;
      row.append(nm);
      body.append(row);
    }
    if (files.length > 3) {
      const more = document.createElement("div");
      more.className = "pl-5.5 text-[10.5px] text-neutral-600";
      more.textContent = `+ ${files.length - 3} weitere`;
      body.append(more);
    }
    const meta = document.createElement("div");
    meta.className = "mt-1 text-[10px] text-neutral-600";
    meta.textContent = item.meta;
    body.append(meta);
    card.append(body);
  }

  // Karten-Aktionen
  card.addEventListener("click", () => actions.onPaste(item.id));
  card.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      actions.onPaste(item.id);
    } else if (e.key === "Delete") {
      e.preventDefault();
      actions.onDelete(item.id);
    }
  });

  return card;
}

// ─── Spalten-Rendering ───────────────────────────────────────────────────
export function renderColumn(
  listEl: HTMLElement,
  items: ItemLite[],
  kind: "text" | "media",
  actions: CardActions,
  nextId?: string
): void {
  listEl.innerHTML = "";
  if (items.length === 0) {
    listEl.append(emptyState(kind));
    return;
  }
  for (const item of items) {
    listEl.append(makeCard(item, actions, nextId));
  }
}
