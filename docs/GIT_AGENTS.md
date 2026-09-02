# Git-Agent-Identitäten (presence-monitor, projektlokal)

Commits tragen eine **explizite Agent-Identität**, damit Git die echte
Autorin/den echten Autor zeigt. **Nie** mit der Standard-/Global-Identität
committen (dort steht z. B. der Mensch `st0rax`), sonst verwischt die Herkunft.

Nicht die Dummy-Tabelle kopieren. Nicht `*@webagent.local`. Nicht
`*@hombot.local`. Nicht `*@bot2bot.local`. Hier gilt **`@presence.local`**.

## Tabelle (offiziell)

| Agent | Git author/committer | Typ |
| --- | --- | --- |
| pflege | `pflege <pflege@presence.local>` | bazaar |
| derfuhrer | `derfuhrer <derfuhrer@presence.local>` | bazaar |
| koordinator | `koordinator <koordinator@presence.local>` | bazaar |
| skeptiker | `skeptiker <skeptiker@presence.local>` | bazaar |
| rust | `rust <rust@presence.local>` | bazaar |
| designer | `designer <designer@presence.local>` | bazaar |
| nachtschicht | `nachtschicht <nachtschicht@presence.local>` | bazaar |
| pseudo-mensch | `pseudo-mensch <pseudo-mensch@presence.local>` | bazaar |

## Benutzung

Es gibt hier **kein** Dummy-`scripts/commit-as-agent.ps1`. Setze die Env
und committe auf dem benannten Zweig:

```bash
export GIT_AUTHOR_NAME='pflege'
export GIT_AUTHOR_EMAIL='pflege@presence.local'
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME"
export GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"
```

Windows analog (`$env:GIT_AUTHOR_NAME` …) oder ein lokaler Wrapper, der
**nur** Namen aus dieser Tabelle akzeptiert.

Push über den System-Credential-Store (keine Secrets anfassen).

## Schirm / Schutz

Nur auf `master` oder einem korrekt benannten Arbeits-Zweig
(`feature|fix|docs|chore|refactor|test/<kurz>`). Andere Zustände ablehnen.

## Legacy-Hinweis

Frühe Commits können noch `st0rax` als Autor tragen. Neue Arbeit nutzt
ausschließlich die Tabelle oben.
