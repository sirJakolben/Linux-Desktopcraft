# Minecraft Shell – Designplan

Stand: 16.10.2026 · Status: **Entwurf aus dem Brainstorming, noch nichts implementiert**

> **Hinweis für KI-Sessions:** Dieser Plan beschreibt, **wie der Nutzer mit den Features interagiert**: Tastenbelegung, Mausaktionen und Aufbau der Oberfläche. **Was die Features sind und tun**, steht in [featurelist.md](featurelist.md). Beide Dateien gemeinsam pflegen: Kommt ein Feature hinzu, steht es in der Featureliste, und seine Bedienung hier.

---

## 1. Grundregeln der Bedienung

- **Win + Taste** = mit der Welt interagieren.
- **Strg + Win + Taste** = Quick Settings, etwas verändern.
- Die Maus bleibt ein **normaler Mauszeiger** und steuert die Kamera nicht (Ausnahmen: Mittelmaustaste und Randpfeile).
- Win-Kombinationen werden vom Programm abgefangen. Sie funktionieren in jeder App (zum Beispiel Win + E für das Inventar).
- Ist der Nutzer ausgetabt (in einer App), pausiert die Welt (siehe Featureliste, Pausieren der Welt).

## 2. Tastenbelegung (Übersicht)

| Taste / Aktion                 | Wirkung                                                                      |
| ------------------------------ | ---------------------------------------------------------------------------- |
| Win + W                        | nach vorn zum nächsten Waypoint                                              |
| Win + S                        | zurück. Im Vollbild: zurück in die Welt                                      |
| Win + A / Win + D              | 90° nach links / rechts drehen. Im Vollbild: zurück in die Welt              |
| Win + E                        | Inventar öffnen (in jeder App)                                               |
| Mittelmaustaste gehalten       | Kamera frei drehen                                                           |
| Randpfeil anklicken            | Ansicht um 90° drehen                                                        |
| Win + Doppelklick auf Boden    | Waypoint setzen                                                              |
| Doppelklick auf Waypoint       | zum Waypoint gehen (verlässt auch die Freisteuerung)                         |
| Strg + Win auf Waypoint        | 4 Seitenpunkte zum Verbinden zeigen                                          |
| Strg + Win + F                 | Freisteuerung einschalten                                                    |
| Strg + Win + Scrollrad         | FOV ändern (ran-/rauszoomen)                                                 |
| Doppelklick auf Wand-Screen    | Screen wird wieder zum 2D-Fenster                                            |
| Win + Drag (2D-Fenster)        | Fenster als Screen in die Welt setzen                                        |
| Win + Klick auf Screen         | Größenpunkte an den Ecken zeigen                                             |
| Strg + Win + Klick auf Screen  | Minus (oben links) und Kreuz (oben rechts) zeigen                            |
| Win + Drag (aus Hotbar)        | Anwendung als Screen in die Welt ziehen                                      |
| Win + S (im 2D-Modus)          | in den Minecraft-Modus wechseln; Fenster schrumpfen in einen Mini-Desktop    |
| Win gehalten                   | Symbolleiste über der Hotbar einblenden                                      |
| Klick auf Mini-Desktop         | Fenster aufgehen lassen, 2D-Modus aktivieren                                 |
| Klick auf Icon mit Glow        | offenes Fenster als Desktop-Fenster öffnen                                   |
| Drag (Icon) auf den Desktop    | Icon verschwindet aus der Hotbar, Fenster erscheint auf dem Desktop          |
| Win + Drag (offenes Fenster)   | Fenster als Screen in die Welt setzen                                        |
| Drag (Fenster/Eintrag) in Hotbar | Fenster wird zum Icon mit Glow                                             |
| Rechtsklick auf Icon/Fenster   | Pop-up mit „Teleport to" (zum Prime-Fenster in der Welt)                     |
| Drag (Item in Kiste/Inventar/Hotbar) | Item physisch in diesen Ordner verschieben                             |
| Drag (Item innerhalb einer Kiste)    | Item frei auf einen anderen Slot legen                                 |

## 3. Kamera und Bewegung

- **Zentrierte Ansicht:** Standard. Win + W/A/S/D bringt die Kamera immer wieder dorthin zurück.
- **Frei drehen:** Mittelmaustaste gedrückt halten.
- **Randpfeile:** Hovert die Maus am Bildschirmrand, erscheint dort ein Pfeil. Ein Klick darauf dreht die Ansicht um 90°. (Das Mitschwenken der Kamera bei gehaltener Win-Taste gibt es nicht.)
- **Waypoints:**
  - Setzen per Win + Doppelklick auf einen Bodenblock. Der Waypoint erscheint als leicht leuchtendes Viereck.
  - Er ist automatisch mit dem Waypoint verbunden, auf dem der Nutzer steht.
  - Win + W läuft zum Waypoint in Blickrichtung, Doppelklick auf einen Waypoint läuft direkt dorthin.
  - Strg + Win auf einem Waypoint zeigt 4 Seitenpunkte. Der Nutzer zieht einen Punkt per Drag auf einen anderen Waypoint und verbindet beide.
- **Freisteuerung:** Strg + Win + F schaltet sie ein, die Steuerung ist dann wie in Minecraft. Ein Doppelklick auf einen Waypoint beendet sie.
- **FOV:** Strg + Win + Scrollrad.

## 4. Bedienung der Screens (Fenster in der Welt)

- **Zurück ins 2D-Fenster:** Doppelklick auf den Wand-Screen macht ihn wieder zum 2D-Fenster. Win + S bringt aus dem 2D-Modus in die Welt.
- **Größe ändern:** Win + Klick auf einen Screen zeigt an den Ecken Punkte. Per Drag vergrößert man den Screen über mehrere Blöcke, nur **gerade (rechteckig)**, nicht über Ecken.
- **Minimieren / Schließen:** Strg + Win + Klick auf einen Screen zeigt statt der Größenpunkte oben links ein **Minus** und oben rechts ein **Kreuz**.
  - Minus: das Fenster wandert als Icon in die Hotbar.
  - Kreuz: das Fenster wird geschlossen; seine Wand-Screens werden zu Verknüpfungen mit letztem Screenshot (siehe Abschnitt 5).
- **Umsetzen:** Aus der Hotbar zieht man ein Fenster per Win + Drag auf eine andere Wand.
- **Platzieren:** Beim Ziehen über einen Block erscheint eine **Vorschau** (heller Rahmen wie das Blockziel in Minecraft). Beim Loslassen entsteht ein Screen auf **einem Block (1×1)**.
- **Klicks:** Mausklicks auf einen Screen werden in Fensterkoordinaten umgerechnet und an das echte Fenster weitergegeben.

## 5. Oberfläche (Hotbar, Inventar, Kiste, Portal)

**Hotbar** (unten am Bildrand, im Look der Minecraft-Hotbar)

- Die Hotbar ist **immer sichtbar** und ein eigener Ordner im Datei-Explorer.
- Mitte: Slots für offene und minimierte Programme, standardmäßig 9. Offene Programme erscheinen mit dem **Enchantment-Glow** (siehe „Offene Anwendungen" unten). Entweder wächst die Leiste mit der Zahl der Einträge, oder sie hat von Anfang an mehr (offen).
- **Standard im 2D-Modus:** Es ist **nur die Hotbar sichtbar** (die Taskleiste mit den offenen Anwendungen). Dazu kommen vielleicht **PC-Informationen in der Ecke**: Suchleiste, Ausschalten-Knopf und Uhrzeit.
- **Win gehalten:** Über der Hotbar erscheint eine **Symbolleiste** mit kleinen Symbolen:
  - **Nether-Portal-Symbol:** Weltenwechsel.
  - **Kistensymbol:** öffnet den Datei-Explorer.
  - **Symbol „Offene Anwendungen":** Klick öffnet ein Fenster mit der Liste aller offenen Anwendungen (Aussehen offen).
  - Weitere Symbole wie ein **Snipping-Tool**. Die Leiste ist **anpassbar**: Der Nutzer kann Symbole hinzufügen und entfernen.
- Jedes Item lässt sich per Win + Drag in die Welt ziehen.
- Offen: Ob das Einstellungssymbol in diese Leiste gehört, und wie die Oberfläche im Minecraft-Modus aussieht.

**Inventar** (Win + E)

- Öffnet sich in jeder App als Raster im Minecraft-Inventar-Stil (zum Beispiel 9×3). Die Items sind die Dateien, Verknüpfungen und Ordner im Inventar-Ordner.

**Kisten**

- Jeder Ordner erscheint als Kiste, im eigenen Datei-Explorer überall mit Kistensymbol statt Ordnersymbol.
- **Rechtsklick** auf eine Kiste öffnet sie als Raster mit ihrem Inhalt.
- Der Explorer ist ein **Custom File Explorer**. Weitere Ideen des Nutzers zu Aussehen, Verhalten und Optionen kommen hier dazu.
- **Items verschieben:** Ein Item per Drag von einer Kiste in Inventar, Hotbar oder eine andere Kiste verschiebt die Datei oder den Ordner **physisch** im Dateisystem.
- **Frei platzieren:** Innerhalb einer Kiste lassen sich Items wie in Minecraft frei auf andere Slots ziehen. Die Anordnung ist nicht an die Namenssortierung gebunden und wird gespeichert.

**Offene Anwendungen** (im Hotbar- und Inventar-Ordner)

- Im Explorer zeigen der Hotbar- und der Inventar-Ordner eine **eigene Sektion „Offene Anwendungen"**. Die Einträge sehen aus wie Dateien, haben aber den Enchantment-Glow.
- **X oben rechts** am Icon: Klick schließt die Anwendung.
- **Hover:** Ein Mini-Tooltip zeigt eine Live-Vorschau dessen, was in der Anwendung passiert.
- **Mehrere Instanzen:** Jede Instanz ist ein eigenes Item (zwei Word-Fenster = zwei Items). Sie lassen sich innerhalb von Hotbar und Inventar frei verschieben, aber **nicht in Kisten**.
- **Klick auf das Icon** öffnet das Fenster als normales 2D-Fenster auf dem Bildschirm (Desktop-Ansicht).

**2D-Modus und Mini-Desktops**

- Der **2D-Modus** funktioniert wie ein normaler Desktop: mehrere Fenster auf einem Bildschirm.
- **Wechsel in die Welt:** Win + S im 2D-Modus wechselt in den Minecraft-Modus (zuvor genannt: auch A und D, noch zu klären). **Win + W macht im 2D-Modus nichts**; es navigiert nur in der Minecraft-Welt.
- **Mini-Desktops wirken nicht auf die Welt:** Sie sind reine 2D-Verwaltung. Einzelne Fenster kommen nur per **Win + Drag** in die Welt und per **Doppelklick auf den Wand-Screen** zurück ins 2D-Fenster. Das ist der **einzige Weg** zwischen 2D-Fenster und Wand-Screen.
- **Animation:** Die 2D-Fenster schrumpfen kurz zu einem **Mini-Desktop** unten links. Er zeigt die Fenster verkleinert und bleibt beim Laufen sichtbar.
- **Zurück:** Klick auf den Mini-Desktop lässt die Fenster wieder aufgehen und aktiviert den 2D-Modus.
- **Zweiter Mini-Desktop:** unten rechts, für den nächsten Fenstersatz.
- **Mindestens zwei Fenster:** Ein Mini-Desktop entsteht nur bei zwei oder mehr offenen Fenstern auf dem Bildschirm. Ein einzelnes Fenster wird beim Wechsel zum Icon in der Hotbar.
- **Beispiel:**
  1. Word und PowerPoint sind im 2D-Modus offen. Win + S: beide schrumpfen unten links.
  2. Spotify und Discord werden geöffnet. Win + S: beide schrumpfen unten rechts.
  3. Zeigt der Bildschirm später PowerPoint und Spotify: PowerPoint (identisch mit dem im ersten Mini-Desktop) geht dorthin. Es bleibt nur Spotify übrig, und ein einzelnes Fenster wird zum Icon in der Hotbar (falls es dort noch kein Icon hat).
  4. Zeigt der Bildschirm PowerPoint, Spotify und Discord: PowerPoint geht in den ersten Mini-Desktop, Spotify und Discord (zwei übrige Fenster) in den zweiten.
- **Identisch** heißt: dieselbe Anwendung mit demselben Dokument oder Projekt. Ein anderes PowerPoint-Projekt gilt als anderes Fenster und kommt nicht in den alten Mini-Desktop.
- **Kein Platz frei:** Die Fenster werden zu Icons in der Hotbar.
- **Mini-Desktop schließen:** Strg gedrückt halten und über den Mini-Desktop hovern: Oben rechts erscheint ein **X**. Ein Klick löscht den Mini-Desktop. Die Fenster darin bleiben offen.

**Die drei Ansichten eines offenen Fensters (Drag & Drop)**

- **Liste „Offene Anwendungen":** Das Symbol in der Hotbar öffnet ein Fenster mit allen offenen Anwendungen. Ein Klick auf einen Eintrag öffnet ihn auf dem Desktop. Einträge lassen sich ziehen:
  - auf den **Desktop**: Fenster-Ansicht,
  - mit gehaltener **Win-Taste** auf den Desktop oder in die Welt: Wand-Ansicht (Screen in der Welt),
  - in die **Hotbar** (oder das Inventar): Icon-Ansicht mit Glow.
- **Aus der Hotbar ziehen:** Das Glow-Icon lässt sich auf den Desktop ziehen (es verschwindet dann aus der Hotbar, das Fenster erscheint) oder mit Win in die Welt.
- **Kombinierbar:** Dasselbe Fenster kann gleichzeitig als Icon, als Desktop-Fenster und als beliebig viele Wand-Screens existieren, oder nur in einigen dieser Ansichten.
- **Prime-Fenster und Teleport:** Der zuletzt benutzte Wand-Screen eines Fensters ist das Prime-Fenster. Rechtsklick auf das Icon oder das Fenster öffnet ein Pop-up mit dem Knopf **„Teleport to"**. Er bringt den Nutzer an die Position, an der das Fenster in der Welt geöffnet wurde.
- **Fenster schließen:** Alle Wand-Screens des Fensters werden zu Verknüpfungen mit dem letzten Screenshot. Ein Klick darauf startet die Anwendung neu und aktualisiert den Screen.

**Nether-Portal**

- Klick auf das Portal-Symbol in der Hotbar öffnet die **Weltenauswahl** (Raster mit Vorschaubild je Welt). Ein Klick auf eine Welt wechselt dorthin, zur letzten Position, mit violetter Überblendung.
- **Neben jeder Welt** ein zweiter Knopf: startet die Welt im echten Minecraft (wie ein Launcher). Aussehen des Knopfs offen.

**Einstellungen**

- Das Einstellungssymbol öffnet die Einstellungen, zum Beispiel für Mobs (Haustiere, benannte, passive, Liste) und die KI-Stufe.

**Weltupdate-Banner**

- Oben links erscheint nach dem Beenden von Minecraft ein Banner mit Fortschrittsanzeige und am Ende einem Haken („gespeichert").

## 6. Offene Fragen zur Bedienung

1. **Hotbar:** 9 Slots, die wachsen, oder von Anfang an mehr? Was öffnet das Kistensymbol daneben?
1a. **Kisten / Explorer:** Wo speichert das Programm die freien Item-Positionen (versteckte Datei im Ordner, zentrale Datenbank, erweiterte Dateiattribute)? Was passiert mit Ordnern, die der Nutzer außerhalb des eigenen Explorers verändert? Genaues Aussehen und Optionen des Explorers (kommen vom Nutzer).
2. **Doppelklicks:** Brauchen Doppelklick auf Waypoint und auf Screen die Win-Taste oder gehen sie ohne? Was bewirkt ein einfacher Klick ohne Win auf einen fernen Screen (Fenster bedienen, nähertreten)?
3. **Waypoints:** Was passiert beim Laufen aus einer unverbundenen Seite (frei weiterlaufen oder nichts)? Mehrere Verbindungen pro Seite erlaubt? Darf ein neuer Waypoint beliebig weit vom aktuellen liegen?
4. **Randpfeile:** an allen vier Rändern oder nur links und rechts?
5. **Freisteuerung:** Normales WASD ohne Win und Maus als Kamera mit Mauszeiger-Sperre? Wirkt Win + Doppelklick auf den Boden weiter?
6. **Screens:** Verhalten von Inhalt und Auflösung beim Aufziehen (die Wand begrenzt das Aufziehen, Blöcke ohne Hitbox werden ignoriert). Bleibt der Wandplatz minimierter Fenster als leerer Rahmen erhalten? Programme haben oft eine Mindestgröße und vertragen kein kleines Quadrat: Wird das Fenster in einen 1×1-Screen hineinskaliert (mit Rand), oder passt sich die Fenstergröße dem Screen an?
7. **Banner:** nur Fortschritt und Haken, oder auch eine Zusammenfassung der Änderungen?
8. **Mobs-Einstellungen:** Welche Stufen genau (Haustiere, benannte, passive, Liste)?
9. **Neue Fenster, Dialoge, Menüs:** Wo erscheint ein neu geöffnetes Fenster oder ein Dialog (Vollbild, nächste freie Wand, zuerst in der Hotbar)? Wie werden Kontextmenüs, Dropdowns und Tooltips gezeigt, die über den Screen hinausragen?
10. **Tastaturfokus:** Welcher Screen bekommt die Tastatureingaben (zuletzt angeklickt, unter dem Mauszeiger)? Wie gibt ein Programm, das alle Tasten abfängt (Spiel, Terminal), die Steuerung zurück an die Welt (Fluchttaste)? Vorbilder: Waylandcraft (`G`, `Alt+Q`), SimulaVR (Super+Z).
12. **Drei Ansichten:** Bleibt das Glow-Icon in der Hotbar, wenn man darauf klickt und das Desktop-Fenster aufgeht (nur beim Ziehen verschwindet es)? Wie sieht das Symbol „Offene Anwendungen" aus, und wo sitzt es genau? Was geschieht, wenn ein Wand-Screen und ein Desktop-Fenster gleichzeitig dasselbe Fenster zeigen (gleiche Größe, wer bekommt die Eingabe)? (Entschieden: Der 2D-Modus ist ein eigener Modus, keine Ebene über der Welt. „Teleport to" ist ausgegraut, solange es keinen Wand-Screen gibt.)
13. **Mini-Desktops:** Was passiert mit den Fenstern, wenn man einen Mini-Desktop schließt (bleiben sie offen als Icon in der Hotbar)? Wechseln auch Win + A und Win + D in die Welt (aktuell nur Win + S)? Ist der Doppelklick auf einen Wand-Screen ein normales Fenster oder Vollbild? Wie werden die Fenster im 2D-Modus angeordnet und was bleibt von der Anordnung nach dem Zurückholen erhalten? Gibt es nur zwei Plätze, oder später mehr?11. **Zentrierte Kamera:** Woran ist sie ausgerichtet (waagerechter Blick entlang einer Himmelsrichtung)?
