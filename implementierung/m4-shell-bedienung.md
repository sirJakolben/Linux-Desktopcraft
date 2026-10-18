# M4: Shell-Bedienung

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R27.
- Detaildesigns fehlen: Hotbar, Inventar, Explorer, Speichern der Item-Positionen.
- Fragen an den Nutzer fehlen: D1a, F9.

**Größe:** 3–6 Wochen, dazu der eigene Datei-Explorer (noch nicht geschätzt). Risiko mittel.
**Voraussetzung:** M3.
**Features:** [features.md](../features.md) §5 (Shell-Elemente), drei Ansichten und Mini-Desktops in §4, eigene Daten in §6. **Bedienung:** [bedienung.md](../bedienung.md) ab „Hotbar und Symbolleiste".

## Ziel

Die Shell-Elemente aus §5 umsetzen: Hotbar, Symbolleiste, Inventar, Kisten, eigener Explorer, offene Anwendungen, Nether-Portal, Einstellungen. Dazu Waypoints setzen und verbinden sowie das Layout pro Welt speichern.

## Kern: darauf kommt es an

1. **Echte Ordner, physisches Verschieben, kein Datenverlust.** Jedes Ziehen eines Items verschiebt Dateien wirklich. Fehlerfälle müssen sicher behandelt werden:
   - Verschieben auf eine andere Partition ist Kopieren plus Löschen.
   - Namenskonflikte und fehlende Rechte.
   - Es braucht Rückgängig oder einen Papierkorb.
2. **Overlay-Ebene des Compositors:** Die Hotbar ist immer sichtbar, das Inventar ist in jeder App erreichbar, auch über Vollbild-Programmen.
3. **Eigener Explorer** als eigenes Teilprojekt. Kistensymbol statt Ordnersymbol überall; freie Item-Positionen werden gespeichert.

## Zu entscheiden

| Frage | Quelle |
| ----- | ------ |
| Speicherort der Item-Positionen, Änderungen außerhalb des Explorers | D1a; R27 |
| Echte Kisten in der Welt mit Ordnern verbunden? | F9 |
| Drei Ansichten | D12 |
| Mini-Desktops (auch: Wechseln Win + A/D in die Welt?) | D13 |
| Symbolleiste und Oberfläche im Minecraft-Modus | D14 |
| Welche Ordner sind Hotbar und Inventar (Pfad), und wie werden sie angelegt? | noch nicht gefragt |
| Wie arbeitet die Suche (Dateiindex, Programme)? | noch nicht gefragt |

## Technische Grundlagen

- **Programme:** freedesktop-`.desktop`-Dateien (Desktop Entry Specification), Symbole über das Icon-Theme.
- **Glow:** Die Glint-Textur kommt aus den Assets (Pfad für 26.x prüfen, zuletzt `textures/misc/enchanted_glint_item.png`).
- **Offene Anwendungen:** ein Item pro Toplevel ohne Elternfenster; Dialoge zählen nicht.
- **Vorschau:** Ein minimiertes Fenster bekommt keine Frame-Callbacks. Für die Hover-Vorschau bekommt es kurz wieder welche.
- **Weltenauswahl:**
  - Vorschaubild ist die `icon.png` im Weltordner.
  - Die letzte Position steht in `level.dat` (`Player.Pos`, `Player.Dimension`).
- **Dateisystem:**
  - Ordner mit inotify überwachen.
  - Papierkorb nach der freedesktop Trash Specification.
- **Drag & Drop zwischen Explorer und Programmen:** Der Compositor tritt selbst als `wl_data_device`-Quelle auf.
- **Datei-Dialoge anderer Programme** zeigen weiter den System-Dialog. Ein eigenes `xdg-desktop-portal`-Backend für den FileChooser wäre eine spätere Option.

## Unbedingt beachten

- **Super-Taste:** unter Linux unkritisch, weil die Shell selbst alle Eingaben steuert (keine Konflikte mit Systemkürzeln).

- **Datensicherheit vor Optik:**
  - Erst kopieren und prüfen, dann löschen.
  - Abbrüche sauber behandeln.
  - Nie stillschweigend überschreiben.
- **Fremde Änderungen:** Andere Programme ändern dieselben Ordner. Der Explorer muss Änderungen von außen sofort sehen und dafür gespeicherte Positionen korrigieren.
- **Offene Anwendungen nie als Dateien anlegen.**
- **Overlay vor Vollbild:** Die Hotbar darf Vollbild-Programme nicht verlangsamen; Direct Scanout geht mit Overlay nur eingeschränkt.

## Abnahme (ohne Codelesen prüfbar)

- **Verschieben:** Ein Item aus einer Kiste in die Hotbar ziehen. Die Datei liegt danach nachweislich im Hotbar-Ordner, sichtbar auch in einem fremden Dateimanager.
- **Rückgängig:** Ein Verschieben auf eine andere Partition lässt sich rückgängig machen.
- **Positionen:** Freie Positionen in einer Kiste bleiben nach einem Neustart erhalten.
- **Inventar:** Es öffnet sich über einem Vollbild-Programm.
- **Weltwechsel:** Der Wechsel über das Portal landet an der letzten Position.

## Referenzen

- freedesktop: Desktop Entry Specification, Icon Theme Specification, Trash Specification
- BumpTop (Apache 2.0): Ideengeber für Ordner als Objekte
