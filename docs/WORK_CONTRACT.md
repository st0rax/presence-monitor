# Arbeitsleitfaden (Bazaar, presence-monitor)

Dieser Text ersetzt keinen Auftrag. Er ist ein **neutraler Leitfaden** für die
freiwillige Zusammenarbeit mehrerer Agents an **presence-monitor**. Dummy
erklärt das Wie generisch — hier gelten **dieses Repo** und `AGENTS.md`.

Teilnahme ist **freiwillig**. Richtung und Tempo bestimmt der Mensch. Ein Agent,
der etwas lieber nicht übernimmt, sagt das klar statt halbherzig. Der Leitfaden
**ermächtigt** Agents, er zwingt sie nicht gegen den Menschen.

## Claim

- Nur in `docs/TASKBOARD.json`: `status=claimed`, `owner`, `branch`,
  `claimed_at`.
- Eine JSON-`id`, ein Entwickler.
- **G-001 ist kein Claim** und keine TASKBOARD-Zeile.
- Markdown-Tafeln nicht anlegen und nicht nachziehen.

## Eine Sache

Zweig von `master`: `feature|fix|docs|chore|refactor|test/<id>-<kurz>`.
Eine Sache pro Zweig, klein halten. `master` immer grün.

## Kanten

`depends_on` = blockiert, bis alle Vorgänger `done` sind. Verkettete Aufgaben
**nie** parallel ziehen.

## Verifikation

Dieses Repo **ist** Rust. Nimm das `verification`-Feld der Zelle, typisch:

- `cargo test`
- `cargo test --lib`
- CI (`.github/workflows/ci.yml`) — Default-Branch ist `master`

Eine Behauptung ohne Beleg gilt nicht als fertig.

## Abnahme / Abnahme-Inspektor

Ein Task gilt als **abgenommen**, wenn die Checkliste vollständig ist:

- [ ] `AGENTS.md` und `START_HERE.md` gelesen
- [ ] Claim vollständig (`owner` / `branch` / `claimed_at`)
- [ ] Zellen-`verification` erfüllt, Beleg in `proof_path`
- [ ] Definition-of-Done erfüllt
- [ ] keine Secrets / Keys / Dummy-HomBot-webagent-Patches
- [ ] PR gegen `master` sauber; Working Tree nach der Arbeit klar

**Rückgabe:** Fehlt etwas, geht der Task mit einer konkreten Mängelliste zurück.
Bei grobem Ausreißer: Zelle auf `free`, Stand rückwärts per neuem Commit.

## Ehrlichkeit

Ergebnisse unterscheiden klar zwischen **geprüft**, **wahrscheinlich**,
**unklar** und **blockiert**. „Fertig“ ohne Beleg ist keins davon.

## Identitäten

Commits tragen `docs/GIT_AGENTS.md` (`*@presence.local`). Nicht
`*@hombot.local`, nicht `*@webagent.local`, nicht `*@bot2bot.local`,
nicht die Dummy-Tabelle.
