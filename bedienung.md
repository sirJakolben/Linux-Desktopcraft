# Bedienung

Wie der Nutzer die Features aus [features.md](features.md) bedient. „Win" = Super-Taste.
Inhalt: Grundregeln · Tasten · Kamera und Waypoints · Screens · Hotbar und Symbolleiste · Inventar, Kisten, offene Anwendungen · Drei Ansichten · 2D-Modus · Portal, Einstellungen, Banner · Offene Fragen D1a–D15

## Grundregeln

- **Win + Taste** = mit der Welt interagieren. **Strg + Win + Taste** = Quick Settings, etwas verändern.
- Die Maus bleibt ein normaler Zeiger und steuert die Kamera nicht (Ausnahmen: Mittelmaustaste, Randpfeile).
- Win-Kombinationen fängt die Shell ab; sie wirken in jeder App.

## Tasten

| Eingabe | Wirkung |
| ------- | ------- |
| Win + W | nach vorn zum nächsten Waypoint in Blickrichtung |
| Win + S | zurück; im Vollbild: zurück in die Welt; im 2D-Modus: in den Minecraft-Modus |
| Win + A / D | 90° links / rechts drehen; im Vollbild: zurück in die Welt |
| Win + E | Inventar öffnen (in jeder App) |
| Win gehalten | Symbolleiste über der Hotbar |
| Mittelmaustaste gehalten | Kamera frei drehen |
| Randpfeil anklicken | Ansicht um 90° drehen |
| Win + Doppelklick auf Boden | Waypoint setzen |
| Doppelklick auf Waypoint | dorthin gehen; beendet auch die Freisteuerung |
| Strg + Win auf Waypoint | 4 Seitenpunkte zum Verbinden |
| Strg + Win + F | Freisteuerung ein |
| Strg + Win + Scrollrad | FOV |
| Win + Drag (2D-Fenster, Hotbar-Item, Listeneintrag) | als Screen in die Welt setzen |
| Doppelklick auf Wand-Screen | wird wieder 2D-Fenster |
| Win + Klick auf Screen | Größenpunkte an den Ecken |
| Strg + Win + Klick auf Screen | Minus (oben links), Kreuz (oben rechts) |
| Klick auf Icon mit Glow | Fenster als Desktop-Fenster öffnen |
| Drag (Icon) auf den Desktop | Icon verschwindet aus der Hotbar, Fenster erscheint auf dem Desktop |
| Drag (Fenster/Eintrag) in die Hotbar | wird Icon mit Glow |
| Rechtsklick auf Icon oder Fenster | Pop-up „Teleport to" (zum Prime-Fenster) |
| Klick auf Mini-Desktop | Fenster gehen auf, 2D-Modus |
| Drag (Item) in Kiste/Inventar/Hotbar | Item physisch in diesen Ordner verschieben |
| Drag (Item) innerhalb einer Kiste | auf anderen Slot legen |

## Kamera und Waypoints

- Win + W/A/S/D bringt die Kamera immer in die zentrierte Ansicht zurück. Ein Mitschwenken bei gehaltener Win-Taste gibt es nicht.
- Randpfeile: Hovert die Maus am Bildschirmrand, erscheint dort ein Pfeil (D4).
- Ein gesetzter Waypoint erscheint als leicht leuchtendes Viereck. Verbinden: einen der 4 Seitenpunkte per Drag auf einen anderen Waypoint ziehen.
- Freisteuerung: Steuerung wie in Minecraft (D5).

## Screens

- **Größe:** Eckpunkte ziehen, nur gerade (rechteckig), nicht über Ecken.
- **Minus:** Fenster wandert als Icon in die Hotbar. **Kreuz:** schließt das Fenster.
- **Umsetzen:** aus der Hotbar per Win + Drag auf eine andere Wand.
- **Platzieren:** Beim Ziehen über einen Block erscheint ein heller Rahmen wie das Blockziel in Minecraft; Loslassen erzeugt den 1×1-Screen.

## Hotbar und Symbolleiste

- Unten am Bildrand. In der Mitte die Slots; offene Programme mit Glow.
- 2D-Modus: nur die Hotbar sichtbar, dazu vielleicht PC-Infos in der Ecke (Suchleiste, Ausschalten, Uhrzeit).
- Symbolleiste (Win gehalten): Nether-Portal (Weltenwechsel), Kiste (Explorer), „Offene Anwendungen" (Liste in einem Fenster), weitere wie Snipping-Tool. Der Nutzer fügt Symbole hinzu und entfernt sie.

## Inventar, Kisten, offene Anwendungen

- **Inventar:** Raster im Stil des Minecraft-Inventars (z. B. 9×3); Items = Inhalt des Inventar-Ordners.
- **Kiste:** Rechtsklick öffnet sie als Raster mit ihrem Inhalt.
- **Offene Anwendung:** X oben rechts am Icon schließt die Anwendung. Hover zeigt die Live-Vorschau als Mini-Tooltip.

## Drei Ansichten

- Liste „Offene Anwendungen": Klick auf einen Eintrag öffnet ihn auf dem Desktop. Ziehen auf den Desktop → Fenster; mit Win auf Desktop oder in die Welt → Wand-Screen; in Hotbar oder Inventar → Icon.
- „Teleport to" ist ausgegraut, solange das Fenster keinen Wand-Screen hat.

## 2D-Modus

- Win + W macht im 2D-Modus nichts; es navigiert nur in der Welt.
- In die Welt kommt ein Fenster nur per Win + Drag, zurück nur per Doppelklick auf den Wand-Screen.
- Erster Mini-Desktop unten links, zweiter unten rechts.
- Mini-Desktop schließen: Strg halten und darüber hovern, dann X oben rechts klicken.

## Portal, Einstellungen, Banner

- **Portal-Symbol:** öffnet die Weltenauswahl (Raster mit Vorschaubild je Welt). Klick auf eine Welt wechselt dorthin, mit violetter Überblendung. Daneben je Welt ein Knopf, der sie im echten Minecraft startet (D15).
- **Einstellungssymbol:** öffnet die Einstellungen (Mobs, KI-Stufe; D14).
- **Weltupdate-Banner:** oben links nach dem Beenden von Minecraft; Fortschritt, am Ende Haken („gespeichert").

## Offene Fragen

| Nr. | Frage                                                                                                                                                                                                                                                                                                                                                                           |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D1a | Kisten/Explorer: Wo werden freie Item-Positionen gespeichert (versteckte Datei im Ordner, zentrale Datenbank, erweiterte Dateiattribute)? Was passiert mit Ordnern, die außerhalb des eigenen Explorers verändert werden? Aussehen und Optionen des Explorers (kommen vom Nutzer).                                                                                              |
| D2  | Brauchen Doppelklick auf Waypoint und auf Screen die Win-Taste? Was bewirkt ein einfacher Klick ohne Win auf einen fernen Screen (bedienen oder nähertreten)?                                                                                                                                                                                                                   |
| D3  | Waypoints: Was passiert beim Laufen aus einer unverbundenen Seite (frei weiter oder nichts)? Mehrere Verbindungen pro Seite? Darf ein neuer Waypoint beliebig weit entfernt liegen?                                                                                                                                                                                             |
| D4  | Randpfeile an allen vier Rändern oder nur links und rechts?                                                                                                                                                                                                                                                                                                                     |
| D5  | Freisteuerung: normales WASD ohne Win und Maus als Kamera mit Zeigersperre? Wirkt Win + Doppelklick auf den Boden weiter?                                                                                                                                                                                                                                                       |
| D6  | Screens: Inhalt und Auflösung beim Aufziehen? Bleibt der Wandplatz minimierter Fenster als leerer Rahmen? Programme mit Mindestgröße: Fenster in den 1×1-Screen hineinskalieren (mit Rand) oder Fenstergröße an den Screen anpassen?                                                                                                                                            |
| D7  | Banner: nur Fortschritt und Haken oder auch eine Zusammenfassung der Änderungen?                                                                                                                                                                                                                                                                                                |
| D9  | Wo erscheint ein neues Fenster oder ein Dialog (Vollbild, nächste freie Wand, zuerst in der Hotbar)? Wie werden Kontextmenüs, Dropdowns und Tooltips gezeigt, die über den Screen hinausragen?                                                                                                                                                                                  |
| D10 | Tastaturfokus: Welcher Screen bekommt die Tasten (zuletzt angeklickt, unter dem Zeiger)? Wie gibt ein Programm, das alle Tasten abfängt (Spiel, Terminal), die Steuerung zurück (Fluchttaste)? Vorbilder: Waylandcraft (`G`, `Alt+Q`), SimulaVR (Super+Z).                                                                                                                      |
| D11 | Woran ist die zentrierte Kamera ausgerichtet (waagerechter Blick entlang einer Himmelsrichtung)?                                                                                                                                                                                                                                                                                |
| D12 | Drei Ansichten: Bleibt das Glow-Icon in der Hotbar, wenn ein Klick das Desktop-Fenster öffnet (nur Ziehen entfernt es)? Aussehen und genaue Position des Symbols „Offene Anwendungen"? Zeigen Wand-Screen und Desktop-Fenster gleichzeitig dasselbe Fenster: welche Größe, wer bekommt die Eingabe?                                                                             |
| D13 | Mini-Desktops: Bleiben die Fenster beim Schließen eines Mini-Desktops als Icon in der Hotbar? Wechseln auch Win + A und Win + D in die Welt (eine frühere Fassung nannte A/S/D)? Wie werden Fenster im 2D-Modus angeordnet, was bleibt nach dem Zurückholen erhalten? Später mehr als zwei Plätze? |
| D14 | Gehört das Einstellungssymbol in die Symbolleiste? Wie sieht die Oberfläche im Minecraft-Modus aus?                                                                                                                                                                                                                                                                             |
| D15 | Aussehen des Knopfs „in Minecraft starten".                                                                                                                                                                                                                                                                                                                                     |
