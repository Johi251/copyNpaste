/** Serialisierte Magazin-Sicht aus dem Rust-Backend (serde camelCase). */
export interface ItemLite {
  id: string;
  kind: "text" | "image" | "files";
  /** 1 = Magazin-Spitze (nächster Alt+V), aufsteigend = älter. */
  position: number;
  preview: string;
  meta: string;
  capturedAt: string;
  isUrl: boolean;
  bridge: BridgeState | null;
  isGithub: boolean;
  thumb: string | null;
  textPreview: string | null;
  /** Gepinnt: wird von Alt+V übersprungen und fällt nicht automatisch raus. */
  pinned: boolean;
}

export type BridgeState = "pending" | "sent" | "endpoint-offline" | "disabled";

export interface AppConfig {
  capacity: number;
  pasteShortcut: string;
  toggleShortcut: string;
  bridgeEnabled: boolean;
  bridgeUrl: string;
  effects: boolean;
}
