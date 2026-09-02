# GOALS — der Nordstern

> **Ein Ziel für alle.** Richtung bestimmt der Mensch. Diese Datei hält sie
> im Repo, damit jeder Agent denselben Fixpunkt liest — ohne Chat-Gedächtnis.
> **G-001 ist kein TASKBOARD-Eintrag. Niemand claimt G-001.**

## G-001 — Hauptziel

> **presence-monitor in diesem Repo soll FERTIG werden:** ein dokumentierter,
> ehrlicher, bazaar-betriebener Ping-basierter Präsenz-Monitor (OnePlus 9 Pro)
> mit Mikrofon-Verifikation, TTS-Begrüßung und Panikmodus, den ein neuer Agent
> allein aus den Dateien weiterführen kann.

- **Was:** „Bin ich zuhause?" via ARP-MAC und Ping des OnePlus 9 Pro plus
  Mikrofon-RMS. Jeder Statuswechsel wird per 30s-Mitschnitt verifiziert.
  Rückkehr (abwesend → anwesend) löst eine hörbare TTS-Begrüßung aus; bleibt
  die Antwort aus, geht das Programm in den **Panikmodus**. Eigenständig —
  keine Verbindung zu webagent/`webagent-rs` oder bot2bot.
- **Wie:** freiwillige kleine Schritte über `docs/TASKBOARD.json` (Kanten =
  `depends_on`; niemand zieht eine verkettete Aufgabe parallel). JSON ist
  die einzige Claim-Tafel.
- **Erfolg:** `cargo test` / `cargo test --lib` grün plus Definition-of-Done
  je Task. Dieses Repo **ist** Rust — das Dummy-Testgate passt hier, es ist
  kein Copy-Paste-Irrtum.

**G-001 ist Nordstern, keine TASKBOARD-Zeile.** Niemand claimt G-001.

## Regeln

- Der Nordstern überschreibt den Bazaar nicht. Er ist das **Was**.
  Der Bazaar (`TASKBOARD.json`, `WORK_CONTRACT.md`) ist das **Wie**.
- `AGENTS.md` und `MISSION.md` bleiben Schutz bzw. aktueller Fokus — sie
  werden nicht in G-001 oder die Tafel „flachgeklappt“.
- Teilnahme bleibt freiwillig (`docs/WORK_CONTRACT.md`).
- Änderungen an G-001 nur durch den Menschen.

## Pflicht-Lese (Reihenfolge)

1. `START_HERE.md` (Einstieg, nicht die einzige Datei)
2. `AGENTS.md` (Schutz)
3. `GOALS.md` (diese Datei)
4. `docs/WORK_CONTRACT.md`
5. `docs/TASKBOARD.json` (Wahrheit; kein Markdown-Board zum Syncen)
