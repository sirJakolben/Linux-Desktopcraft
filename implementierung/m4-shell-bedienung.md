# M4: Shell-Bedienung

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R27.
- Detaildesigns fehlen: Hotbar, Inventar, Explorer, Speichern der Item-Positionen.
- Fragen an den Nutzer fehlen: Designplan, Fragen 1 und 1a; Featureliste, Frage 9.

**Größe laut Featureliste:** 3–6 Wochen, dazu der eigene Datei-Explorer (noch nicht geschätzt). Risiko mittel.
**Voraussetzung:** M3.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M4 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md). Die Bedienung steht im [Designplan](../designplan.md), Abschnitt 5.

## Ziel

Die Bedienoberfläche der Shell:

- Hotbar (Taskleiste) mit Suchleiste, Kisten-, Einstellungs- und Portal-Symbol
- Inventar (Desktop)
- Kisten (Ordner)
- eigener Datei-Explorer
- Nether-Portal (Weltwechsel)
- Einstellungen
- Waypoints setzen und verbinden
- Layout pro Welt speichern

Hotbar, Inventar und Kisten sind **echte Ordner**.

## Kern: darauf kommt es an

1. **Echte Ordner, physisches Verschieben, kein Datenverlust.** Jedes Ziehen eines Items verschiebt Dateien wirklich. Fehlerfälle müssen sicher behandelt werden:
   - Verschieben auf eine andere Partition ist Kopieren plus Löschen.
   - Namenskonflikte und fehlende Rechte.
   - Es braucht Rückgängig oder einen Papierkorb.
2. **Overlay-Ebene des Compositors:** Die Hotbar ist immer sichtbar, das Inventar ist in jeder App erreichbar, auch über Vollbild-Programmen.
3. **Eigener Explorer** als eigenes Teilprojekt. Kistensymbol statt Ordnersymbol überall; freie Item-Positionen werden gespeichert.

## Entschieden

- **Einordnung:** Hotbar = Taskleiste, Inventar = Desktop, Kiste = Ordner, Nether-Portal = Weltenwechsel.
- **Hotbar, Inventar und Kisten:**
  - Sie sind **echte Ordner** und im Explorer sichtbar; Verschieben ist physisch.
  - Jeder Ordner erscheint als Kiste.
  - Die Hotbar ist immer sichtbar, standardmäßig mit 9 Slots.
- **Offene Anwendungen:**
  - Items mit **Enchantment-Glow**, in einer eigenen Sektion im Hotbar- und Inventar-Ordner.
  - Diese Sektion gibt es **nur im eigenen Explorer**, es sind keine echten Dateien.
  - **Jede Instanz ist ein eigenes Item.** Nicht in Kisten verschiebbar.
  - X-Knopf zum Schließen, Live-Vorschau beim Hovern.
  - **Drei unabhängige Ansichten** eines Fensters: Icon (Glow, Hotbar/Inventar), Desktop-Fenster (2D), Wand-Screen (Welt). Beliebig kombinierbar, beliebig viele Wand-Screens pro Fenster. Wechsel per Drag & Drop (Win + Drag = Welt). Klick auf das Icon öffnet das Desktop-Fenster.
  - **Prime-Fenster:** der zuletzt benutzte Wand-Screen. Rechtsklick auf Icon oder Fenster: „Teleport to" zur Position, an der es in der Welt geöffnet wurde.
  - **Schließen:** Wand-Screens werden zu Verknüpfungen mit dem letzten Screenshot; Klick startet die Anwendung und aktualisiert den Screen.
  - **2D-Modus und Mini-Desktops:** Die Shell ergänzt die Welt um eine normale Desktop-Nutzung (mehrere Fenster auf einem Bildschirm). Beim Wechsel in die Welt (Win + A/S/D) schrumpfen die Fenster in einen von **zwei Mini-Desktops** (unten links/rechts, auch in der Welt sichtbar). Ein Mini-Desktop entsteht erst bei **mindestens zwei** offenen Fenstern; ein einzelnes Fenster wird zum Icon. Identische Fenster (gleiche Anwendung und gleiches Dokument) gehören in den Mini-Desktop, in dem sie liegen. Die übrigen kommen in den nächsten freien Mini-Desktop, wenn es mindestens zwei sind, sonst werden sie zu Icons in der Hotbar. Schließen: Strg gehalten, Hover, X oben rechts; die Fenster bleiben offen. Klick auf den Mini-Desktop stellt den 2D-Modus wieder her. Siehe Designplan.
  - **Symbol „Offene Anwendungen"** neben Kisten- und Portal-Symbol: Fenster mit der Liste aller offenen Anwendungen.
- **Kisten:** Items lassen sich frei platzieren wie in Minecraft; die Position wird gespeichert.
- **Nether-Portal:** Weltenauswahl mit Vorschaubild. Der Wechsel geht zur letzten Position (Rückfall: Spawnpunkt), mit violetter Überblendung. Neben jeder Welt sitzt ein Knopf „in Minecraft starten" (Umsetzung in M5).
- **Layout getrennt von der Welt:** Item-Positionen, Fensterlayout und Waypoints stehen nicht in der Weltdatei.
- **Einstellungen** unter anderem für Mobs und die KI-Stufe.

## Zu entscheiden

| Frage | Quelle |
| ----- | ------ |
| Hotbar mit 9 wachsenden Slots oder von Anfang an mehr? Was öffnet das Kistensymbol? | Designplan, Frage 1 |
| Wo werden Item-Positionen gespeichert (versteckte Datei, Datenbank, erweiterte Attribute)? Umgang mit Änderungen außerhalb des Explorers | Designplan, Frage 1a; R27 |
| Sind echte Kisten in der Welt mit Ordnern verbunden? | Featureliste, Frage 9 |
| Drei Ansichten: Bleibt das Icon nach Klick? Desktop als Ebene über der Welt oder eigener Bildschirm? Aussehen und Position des Symbols „Offene Anwendungen"? Verhalten bei Wand-Screen und Desktop-Fenster desselben Fensters? | Designplan, Frage 12 |
| Mini-Desktops: Anordnung im 2D-Modus, Win + W im 2D-Modus, Verbleib der Fenster nach dem Schließen eines Mini-Desktops | Designplan, Frage 13 |
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

## Offene Fragen an den Nutzer

- Designplan, Fragen 1 und 1a.
- Featureliste, Frage 9.
- Vorgaben zu Aussehen und Optionen des Explorers (kommen vom Nutzer).

## Referenzen

- freedesktop: Desktop Entry Specification, Icon Theme Specification, Trash Specification
- BumpTop (Apache 2.0): Ideengeber für Ordner als Objekte
