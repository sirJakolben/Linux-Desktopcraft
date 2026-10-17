# M5: Weltsynchronisierung

**Planungsstand:** grob. Der Ablauf ist klar, aber noch nicht bereit.
- Recherche fehlt: R12, R31.
- Entscheidungen fehlen: T14, T16.
- Fragen an den Nutzer fehlen: Featureliste, Frage 6.

**Größe laut Featureliste:** 2–4 Wochen, Risiko niedrig bis mittel.
**Voraussetzung:** M1. Die Erkennung von Minecraft-Fenster und -Prozess braucht M3.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M5 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md).

## Ziel

- **Weltupdate:** Wird Minecraft beendet, kopiert die Shell die geänderten Teile der Welt in ihren eigenen Cache und baut nur die geänderten Chunks neu. Ein Banner zeigt den Fortschritt und am Ende einen Haken.
- **Welt starten:** Aus der Weltenauswahl startet ein Knopf die gewählte Welt direkt im echten Minecraft.

## Kern: darauf kommt es an

1. **Nie in die Originalwelt schreiben, nur einen konsistenten Stand lesen.** Erst nach dem Prozessende von Minecraft ist die Welt vollständig gespeichert. Davor höchstens den alten Cache zeigen.
2. **Genauer Abgleich auf Chunk-Ebene:** Die Zeitstempel im Kopf jeder Regionsdatei zeigen, welche Chunks sich geändert haben. Nur diese werden neu gebaut.
3. **Start in eine Welt** über einen Launcher, ohne dass die Shell selbst Microsoft-Anmeldedaten verwaltet (T16).

## Entschieden

- **Auslöser** ist das Beenden von Minecraft. Danach wird inkrementell kopiert und neu gebaut, mit Banner und Haken.
- **Linux als Hauptweg:** Minecraft wird unter Linux in der Shell gespielt; Prozess und Fenster sind dort sicher erkennbar.
- **Dual Boot, vorläufige Alternative:** Beim Linux-Start werden die Welten auf der Windows-Partition geprüft und geänderte aktualisiert.
- **Umfang:** nur Singleplayer-Welten, keine Realms, keine Server.
- **Layout getrennt:** Fensterlayout, Item-Positionen und Waypoints stehen getrennt von der Welt.
- **Launcher-Knopf** neben jeder Welt in der Weltenauswahl.

## Zu entscheiden

| Nr. | Frage | Hinweise |
| --- | ----- | -------- |
| T14 | Aufbau des Weltcaches und Erkennung von Änderungen | Region-Zeitstempel, Größe und mtime der Dateien, Inhalts-Hash; wohin der Cache kommt; wie viel Speicher er braucht |
| T16 | Wie Minecraft gestartet wird | einen vorhandenen Launcher steuern (Prism Launcher) oder einen eigenen Launcher bauen (Microsoft-Anmeldung, Spieldateien, Java). Empfehlung nach R31 |
| F6 | Tageszeit nach dem Update | aus dem Spiel übernehmen oder die angehaltene Zeit der Shell behalten (Featureliste, Frage 6) |
| – | Banner: nur Fortschritt oder auch eine Zusammenfassung? | Designplan, Frage 7 |

## Technische Grundlagen

Mit R12 und R31 prüfen.

- **Regionsdatei:** Das zweite 4-KiB-Feld im Kopf enthält 1024 Zeitstempel (Sekunden) der letzten Änderung pro Chunk.
- **Welt in Benutzung:** Eine geöffnete Welt erkennt man an `session.lock`.
- **Minecraft erkennen:** Läuft Minecraft unter unserem Compositor (meist über XWayland), sind Fenster und Prozess bekannt. Die PID kommt über die Client-Credentials des Wayland- bzw. XWayland-Sockets.
- **Start in eine Welt:**
  - Quick Play seit 1.20: `--quickPlaySingleplayer <Weltordner>`.
  - Prism Launcher hat Kommandozeilen-Optionen zum Starten einer Instanz mit Welt (prüfen).
- **Minecraft 26.x unter Linux:** SDL3 statt GLFW; der exklusive Vollbildmodus stürzt unter Wayland ab. Vulkan ist ab den 26.4-Snapshots Standard. Über XWayland läuft es auf NVIDIA robuster.
- **Windows-Partition (NTFS):**
  - nur lesend einbinden
  - Schnellstart oder Ruhezustand von Windows sperrt die Partition; dann nicht einbinden, sondern einen Hinweis zeigen

## Unbedingt beachten

- **Kein Schreibzugriff** auf Weltordner, auch nicht über Sperren oder Zeitstempel.
- **Nie während eines laufenden Spiels kopieren.** Es entstünde ein halber Stand.
- **Spielerposition und Tageszeit** aus `level.dat` erst nach dem Update übernehmen.
- **Fensterlayout** muss gegen geänderte Wände geprüft werden (Featureliste, Frage 10; M3, T12).
- **Fehlertoleranz:** Bricht ein Update ab, bleibt der alte Cache gültig.

## Abnahme (ohne Codelesen prüfbar)

- **Ablauf:** Welt aus der Shell starten, im Spiel einen Block setzen, Minecraft beenden. Das Banner zeigt Fortschritt und Haken, und der Block ist in der Shell sichtbar.
- **Messung:** Bei einer Änderung in einer einzigen Region wird nur diese neu gebaut (Zeit und Zahl der Chunks im Log).
- **Original unverändert:** Prüfsummen der Weltordner vor und nach dem Lauf sind identisch, abgesehen von dem, was Minecraft selbst geschrieben hat.

## Offene Fragen an den Nutzer

- Featureliste, Frage 6.
- Designplan, Frage 7.
- Mit welchem Launcher spielt der Nutzer unter Linux? (Featureliste, Frage 1)
