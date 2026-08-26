# Historische Python-Verhaltensreferenz

> **Status:** Historische Spezifikation, nicht ausführbarer Produktionscode. Der einzige unterstützte Implementierungsweg ist der Rust-Monitor dieses Repositories.

## Zweck der Übernahme

Das frühere Repository `home-presence` enthielt die lesbare Python-Erstreferenz der Präsenzentscheidung. Diese Datei bewahrt ausschließlich die dort festgelegten **Verhaltensregeln**. Sie dient nicht als Vorlage für neue Python-Entwicklung und ersetzt weder `START_HERE.md` noch `config.json` als aktuelle Quelle.

## Übernommener Entscheidungsvertrag

Die Python-Referenz nutzte zwei lokale Signale:

| Signal | Historische Regel | Fehlerverhalten |
|---|---|---|
| Telefon im LAN | Suche nach einer vollständigen MAC-Adresse oder einem OUI-Präfix in `arp -a`; bei gesetztem SSID-Wert zusätzlich SSID-Prüfung über `netsh wlan show interfaces`. | Kein MAC-Filter bedeutet: Telefonsignal deaktiviert. |
| Mikrofon | Etwa eine Sekunde RMS-Sampling; ein Wert **größer** als die Schwelle bedeutet Aktivität. | Nicht verfügbares Mikrofon oder Aufnahmefehler ergeben `0.0`; es werden weder Audio noch Transkripte gespeichert. |

Die historische Formel lautete:

```text
present = (phone_present AND wlan_ssid_ok) OR (mic_rms > mic_threshold)
```

Der Rust-Kern bewahrt die ARP-/SSID-/RMS-Semantik. Zusätzlich besitzt er einen Ping-Pfad sowie Zustandswechsel-Verifikation, TTS und Panikmodus. Diese späteren Rust-Funktionen sind bewusst **nicht** Teil des alten Python-Vertrags.

## Übernommene Umgebungsvariablen

| Variable | Historische Bedeutung | Historischer Standard | Rust-Status |
|---|---|---:|---|
| `PRESENCE_PHONE_MAC` | Vollständige MAC-Adresse oder OUI-Präfix für den ARP-Abgleich | leer | Weiterhin unterstützt; siehe `device.phone_mac_prefix`. |
| `PRESENCE_WLAN_SSID` | Optional notwendige WLAN-SSID für das Telefonsignal | leer | Weiterhin unterstützt; siehe `device.wlan_ssid`. |
| `PRESENCE_MIC_THRESH` | RMS-Schwelle für Mikrofon-Aktivität | `0.01` | Weiterhin unterstützt; siehe `mic.rms_threshold`. |

Die kanonischen aktuellen Konfigurationsnamen und zusätzliche Rust-spezifische Optionen stehen in `config.json`, `README.md` und `src/config.rs`.

## Historische Ausgaben und Bedienung

Die Python-Version kannte einen Einmalmodus, JSON-Ausgabe und periodisches Polling. Ihre Diagnose enthielt mindestens `present`, SSID-/Telefon-Status, RMS-Wert, Schwelle und `voice_detected`. Der Rust-Monitor hat bewusst eine andere CLI und weitergehende Persistenz für Zustandswechsel; neue Integrationen müssen die aktuelle Rust-Schnittstelle verwenden.

## Migrationsentscheid

`home-presence` bleibt nur bis zur bestätigten Archivierung oder Löschung als Git-Historie erhalten. Diese Datei ist die lesbare Verhaltensbrücke; sie verhindert, dass der Python-Bestand allein wegen Schwellen, ARP-/SSID-Semantik oder Fehlverhalten weiter aktiv gehalten werden muss.
