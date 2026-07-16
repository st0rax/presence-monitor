# presence-monitor — Design- & Doku-Konventionen

## Design-Prinzipien

- **Hardware immer hinter einem Trait.** `PhoneProbe`, `MicRecorder`,
  `MicLevelSampler`, `TtsEngine`, `PresenceProbe` — jede neue Hardware-
  Interaktion bekommt eine eigene, injizierbare Trait-Grenze. Das ist der
  Grund, warum der Kern ohne echte Hardware testbar ist; nicht aufweichen.
- **Reine Logik von Hardware trennen.** `state.rs` (Zustandsmaschine) und
  `presence.rs` (Verdict-Berechnung) bleiben frei von direkten Hardware-
  Aufrufen — die kommen ausschließlich über die Traits.
- **Windows-only ist akzeptiert, nicht bekämpft.** SAPI (TTS) und `arp -a`
  sind plattformspezifisch — Portabilität ist explizit kein Ziel (siehe
  `README.md`). Kein `#[cfg]`-Aufwand für andere Plattformen ohne konkreten
  Bedarf.
- **Kein Suite-Framing.** Komplett eigenständig, keine Abhängigkeit zu
  webagent/webagent-rs oder bot2bot.
- **Fallback-Verhalten muss ehrlich sein.** Ein Fallback (z. B. Ping auf
  `127.0.0.1`, wenn das Zielgerät nicht erreichbar ist) darf nicht
  fälschlich als positives Signal zählen — lieber `false`/„unbekannt" als
  ein Fallback-Erfolg, der wie ein echtes Ergebnis aussieht.

## Was NICHT tun

- Keine `unwrap()`-Explosion im Happy-Path — `anyhow` für Fehlerpfade nutzen,
  wie im restlichen Code.
- Keine Hardware-Aufrufe direkt in `state.rs`/`presence.rs`/`monitor.rs` —
  immer über die injizierten Traits.
- Keine stillen Fallbacks, die wie ein echtes Ergebnis aussehen (siehe oben).

## Doku-Richtlinien

- **`START_HERE.md`** — einziger Einstiegspunkt, Status + Architektur +
  Build/Test + offene Punkte. Pflegepflicht: bei jeder strukturellen
  Änderung sofort mitziehen.
- **`MISSION.md`** — aktueller Arbeitsfokus, ändert sich häufiger als
  `START_HERE.md`.
- **`CONVENTIONS.md`** (diese Datei) — Design-Prinzipien + Doku-Organisation.
  Ändert sich selten.
- **`README.md`** — vollständige Funktionsbeschreibung (Panikmodus-Ablauf,
  Logging-Struktur, Config-Tabelle, Autostart). Bleibt die primäre
  Referenz für „wie funktioniert das im Detail" — `START_HERE.md` dupliziert
  das nicht, sondern verweist dorthin.
- **Kein neues Root-`.md` ohne Grund.** Passt der Inhalt in eine der obigen
  Dateien? Dann dort rein.
