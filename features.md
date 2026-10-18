# Features

Was die Shell tut. Bedienung: [bedienung.md](bedienung.md).
Inhalt: 1 Vision · 2 Navigation · 3 Welt und Rendering · 4 Fenster in der Welt · 5 Shell-Elemente · 6 Weltsynchronisierung · 7 Performance · 8 System · 9 Recht · 10 Multiplayer · Offene Fragen F1–F12

## 1 Vision

- Komplette Desktop-Shell als begehbare Minecraft-Welt; der PC startet direkt darin. Kein echtes Minecraft: Die Umgebung wird nur gerendert und ersetzt Hintergrund, Taskleiste und Fensterverwaltung.
- Liest **echte Weltdateien**: gleiche Texturen, Blockmodelle, Beleuchtung, aber nur, was für einen Desktop sichtbar nötig ist.
- Belastet den Rechner im Alltag **kaum**.
- **Ergänzt** den normalen Desktop: Arbeiten wie unter Windows (2D-Modus), jederzeit Wechsel in die Welt.
- Der Nutzer wohnt in einem Haus; an den Wänden hängen Screens mit Programmen, Icons und Ordnern.

## 2 Navigation

- **Waypoints:** Bodenpunkte, zu denen der Spieler automatisch läuft (wie Street View). Sie bilden ein Wegenetz: jeder Waypoint hat 4 Seiten, jede kann mit einem anderen Waypoint verbunden sein (D3). Läuft der Spieler aus einer Seite heraus, geht er zum dort verbundenen Waypoint. Ein neuer Waypoint ist sofort mit dem verbunden, auf dem der Spieler steht. Standardweg, aber nicht der einzige.
- **Kamera:** standardmäßig exakt zentriert und fest ausgerichtet (D11), kehrt nach jeder Bewegung dorthin zurück. Bei Bedarf frei drehbar; Drehen in 90°-Schritten.
- **Freisteuerung:** löst den Spieler von den Waypoints, Bewegung wie in Minecraft als feste Teilmenge (F4). Zurück ins Netz durch Gehen zu einem Waypoint.
- **FOV** einstellbar.

## 3 Welt und Rendering

- **Minecraft-Look:** Texturen, Blockmodelle, Beleuchtung, Weltstruktur wie im Spiel; Texturen und Modelle zur Laufzeit aus der lokal installierten Version.
- **Minimal:** kein Bauen und Abbauen, keine Spielphysik außer der Freisteuerung, nichts Unsichtbares.
- **Behalten:** Sonnenzyklus, Himmel, Wolken, Shader, Wasser- und Blätterbewegung (F7), animierte Texturen (dem Nutzer wichtig).
- **Pause:** Ist der Nutzer in einer Anwendung (F5), stehen Tageszeit, Animationen und Shader. Im Vollbild entstehen keine Frames. Ist die Welt sichtbar (Screen an der Wand), wird nur bei Fensteränderungen neu gezeichnet. Danach läuft die Zeit exakt dort weiter.
- **Umfang:** die ganze Welt, frei erkundbar. Neue Chunks werden nie erzeugt; wo die Welt nicht generiert ist, endet sie. Nur Singleplayer-Welten der Java Edition, keine Realms und Server (zunächst). Immer nur eine Welt geladen.
- **Mobs:** standardmäßig aus. Zuschaltbar in den Einstellungen: Haustiere, alle mit Namensschild, alle passiven oder eine Liste einzelner Mobs (z. B. Villager). Nur minimale Logik (Villager ohne Trades und Workstations, laufen zufällig). KI-Stufe vereinfacht, in den Einstellungen hochstellbar.
- **Renderdistanz** größer als in einem einfachen Viewer. Ausbaustufe: Fern-Detailstufen wie Distant Horizons; zeigen nur geladenes oder besuchtes Gelände, einmal beim Weltupdate berechnet und gespeichert.
- **Mods, Shader, modifizierte Welten** (Ziel, späte Ausbaustufe, Weg offen), mit eigenem, effizientem Renderer (z. B. Rust):
  - **Mod-Adapter:** zieht nur Custom-Blöcke, Texturen und eventuell Custom-Mobs heraus; Rezepte, Blockverhalten und Texturanimationen der Mods fallen weg. Einschätzung (ungeprüft): Blöcke und Texturen aus den Asset-Dateien machbar; Blöcke mit eigenem Darstellungscode und Mods, die das Rendering ändern, kaum.
  - **Shader:** eingebauter Shader, vielleicht in mehreren Versionen: bekannte Minecraft-Shader in eigenen Code übersetzt und verschlankt. Sie bilden wie bei Mods die Liste unterstützter Shader; hat der Nutzer einen davon im echten Minecraft installiert, nutzt die Shell ihn auch. Direkte Kompatibilität mit beliebigen Shader-Packs (OptiFine/Iris) ist kein Ziel.

## 4 Fenster in der Welt

- **Screens:** Programme als echte Fenster mit Live-Textur an den Hauswänden. Desktop-Icons hängen als kleine Screens wie Item-Frames.
- **Persistenz:** Geöffnete Fenster bleiben an der Wand offen und sind aus der Ferne sichtbar: nah scharf und bedienbar, fern kleiner und perspektivisch verzerrt. Eingaben gehen korrekt an das echte Fenster.
- **Vollbild:** Die Welt wird dabei nicht gerendert.
- **Größe:** über mehrere Blöcke, nur rechteckig (keine L-Formen). Die **Wand ist die Grenze** (F10). **Blöcke ohne Hitbox** werden ignoriert und lassen sich überziehen: Blumen, Gras, Fackeln, Töpfe und alles Kleine, Durchlaufbare, das kein voller Block, keine Slab und keine Treppe ist.
- **Platzieren:** Vorschau zeigt den Zielblock; ein neuer Screen belegt 1×1 Block.
- **Drei unabhängige Ansichten** eines offenen Fensters, per Drag & Drop wechselbar, beliebig kombinierbar, keine Pflicht (ein Desktop-Fenster braucht kein Hotbar-Icon):
  1. **Icon:** kleines Icon mit Enchantment-Glow, in Hotbar oder Inventar, nie in Kisten.
  2. **Fenster:** normales 2D-Fenster auf dem Desktop, auch im Vollbild.
  3. **Wand:** Screen an einer Wand der Welt.
- **Mehrere Wand-Screens:** Dasselbe Fenster darf beliebig oft an Wänden hängen. Der zuletzt benutzte ist das **Prime-Fenster**; Teleport dorthin führt zur Position, an der es in der Welt geöffnet wurde.
- **Minimieren:** als Icon in die Hotbar, von dort wieder auf eine Wand oder den Desktop (Umsetzen).
- **Schließen:** Alle Wand-Screens werden zu **Verknüpfungen** zur Anwendung und zeigen statt des Programm-Icons den letzten Screenshot der Sitzung. Klick startet die Anwendung neu und aktualisiert den Screen (z. B. Word-Dokument mit letztem Stand als Vorschau).
- **2D-Modus und Minecraft-Modus:** Im 2D-Modus liegen mehrere Fenster nebeneinander auf einem Bildschirm wie unter Windows. Er ist ein eigener Modus, keine Ebene über der Welt. Der Nutzer wechselt zwischen beiden Modi.
- **2D ↔ Welt nur einzeln:** ein 2D-Fenster in die Welt setzen oder einen Wand-Screen zum 2D-Fenster machen; das ist der einzige Weg zwischen beiden Ansichten.
- **Mini-Desktops:** Beim Wechsel in den Minecraft-Modus schrumpfen die offenen 2D-Fenster in einer kurzen Animation zu einem Mini-Desktop: kleines Rechteck, zeigt die Fenster verkleinert, bleibt beim Laufen sichtbar. Klick darauf lässt die Fenster aufgehen und aktiviert den 2D-Modus. Zwei Plätze: unten links, unten rechts. Mini-Desktops wirken nicht auf die Welt.
  - Entsteht erst ab **zwei** offenen Fenstern auf demselben Bildschirm; ein einzelnes Fenster wird zum Icon in der Hotbar.
  - Ein Fenster, **identisch** mit einem in einem Mini-Desktop (gleiche Anwendung und gleiches Dokument/Projekt; anderes Projekt = anderes Fenster), kommt in diesen Mini-Desktop.
  - Die übrigen kommen in den nächsten freien Mini-Desktop, wenn es mindestens zwei sind. Bleibt eins übrig, wird es zum Icon (falls noch keins existiert). Ist kein Platz frei, werden alle zu Icons.
  - Schließen löscht nur den Mini-Desktop; die Fenster bleiben offen.

## 5 Shell-Elemente

Hotbar = Taskleiste, Inventar = Desktop, Kiste = Ordner, Nether-Portal = Weltenwechsel.

- **Echte Ordner:** Hotbar, Inventar und Kisten sind die Ordner selbst im Dateisystem (keine Kopien oder Verknüpfungslisten), auch im Explorer sichtbar. Ein Item bewegen verschiebt Datei oder Ordner **physisch**.
- **Hotbar:** Look der Minecraft-Hotbar, eigener Ordner, immer sichtbar. Zeigt offene und minimierte Programme; 9 Slots, mehr bei Bedarf (D1). Anwendungen daraus lassen sich als Screen in die Welt setzen.
- **Offene Anwendungen:** Items mit Enchantment-Glow; Klick öffnet das Fenster. Sie liegen in einer eigenen Sektion im Hotbar- und Inventar-Ordner und sehen aus wie Dateien, sind aber keine: Die Sektion gibt es nur im eigenen Explorer. Nur in Hotbar und Inventar (dort frei verschiebbar), nie in Kisten. Mit Schließen-Knopf und Live-Vorschau. Jede Instanz ist ein eigenes Item (zwei Word-Fenster = zwei Items).
- **Übersicht offener Anwendungen:** eigenes Symbol neben Kisten- und Portal-Symbol; zeigt alle offenen Anwendungen in einem Fenster. Von dort öffnen oder in eine der drei Ansichten ziehen.
- **Suche** auf dem PC.
- **PC-Infos und Symbolleiste:** in einer Bildschirmecke Suchleiste, Ausschalten-Knopf, Uhrzeit. Anpassbare Symbolleiste mit Schnellzugriff auf Weltenwechsel, Explorer, offene Anwendungen und weitere Werkzeuge (z. B. Snipping-Tool). Im 2D-Modus ist standardmäßig nur die Hotbar sichtbar, die Symbolleiste nur bei Bedarf.
- **Inventar:** Desktop-Ersatz mit oft gebrauchten Icons und Verknüpfungen, in jeder App erreichbar, schneller Starter; eigener Ordner.
- **Kisten:** Jeder Ordner ist eine Kiste; im eigenen Explorer überall Kistensymbol statt Ordnersymbol. Inhalt frei zwischen Kisten, Inventar und Hotbar verschiebbar (physisch). In einer Kiste Items frei platzierbar wie in Minecraft statt Namenssortierung; Position wird gespeichert.
- **Eigener Datei-Explorer:** komplett neu geschrieben für das Minecraft-System (Kisten, freie Positionen, Hotbar und Inventar als Ordner). Weitere Ideen des Nutzers zu Aussehen, Verhalten und Optionen kommen später.
- **Nether-Portal** (statt Ender Chest): steht für den Weltenordner; wechselt die Welt zur letzten Position dort (Fallback: Spawnpunkt), mit Überblendung.
- **Welt in Minecraft starten:** Knopf neben jeder Welt in der Weltenauswahl (Launcher-Funktion). Gespielt wird unter Linux in der Shell; beim Beenden folgt das Weltupdate (§6).
- **Einstellungen:** zentral, u. a. Mobs und KI-Stufe.

## 6 Weltsynchronisierung

- Jede Java-Welt lädt **ohne manuelle Konvertierung**; es ist dieselbe Welt wie im Spiel.
- **Weltupdate:** Beim Beenden von Minecraft erkennt die Shell das Prozessende (Spiel hat dann vollständig gespeichert), kopiert nur geänderte Regionsdateien in einen eigenen Cache und baut nur geänderte Chunks neu. Die Originalwelt wird nie verändert.
- **Übergang Dual Boot:** Läuft Minecraft unter Windows, läuft die Shell nicht. Dann prüft sie beim Linux-Start die Welten auf der Windows-Partition und aktualisiert geänderte. Hauptweg bleibt Minecraft unter Linux.
- **Banner:** zeigt Fortschritt, am Ende einen Haken (D7).
- **Eigene Daten** (Fensterlayout, Item-Positionen in Kisten, Waypoints) getrennt von der Welt, z. B. eine Layout-Datei pro Welt.

## 7 Performance

- Fertige Meshes werden gespeichert, damit Welten von der SSD in wenigen Sekunden laden. Chunks laden von innen nach außen nach, Nebel kaschiert den Rand.
- Nur die aktive Welt ist geladen, andere nur als Vorschaubild.
- Fenster außerhalb des Blickfelds werden gedrosselt (ca. 1 Bild/s).
- Schätzung (ungemessen): 100–300 MB VRAM bei ca. 12 Chunks Renderdistanz, Fernsicht zusätzlich 1–3 GB.

## 8 System

- Später nötig: Benachrichtigungen, Audio, mehrere Monitore, Energiesparen, **Notfallmodus** (Rückfall auf normalen Desktop bei Absturz). Weitere Grundfunktionen: F11.

## 9 Recht

- Der Kauf von Minecraft ist nur eine Spiellizenz und erlaubt nicht, Mojang-Code in ein anderes Produkt zu übernehmen. Keine Rechtsberatung; EULA und Nutzungsrichtlinien prüfen (F3).
- Verworfen: eine Mod im laufenden Minecraft (das ganze Spiel müsste dauerhaft im Hintergrund laufen).

## 10 Multiplayer (nach dem Release)

- Mehrere Nutzer in derselben Welt; man läuft zu anderen und kann ihre oft geöffneten Fenster ansehen und öffnen. Setzt Netzwerk, Synchronisierung und Rechtekonzept voraus (wer sieht und bedient welche Fenster); nicht ausgearbeitet.

## Offene Fragen

| Nr. | Frage |
| --- | ----- |
| F1 | Linux-Details: Kommen Windows-Programme in die Welt (nur Linux-Apps oder Wine/Proton)? Welcher Launcher unter Linux (offiziell, Prism …)? Distribution: Fedora ist installiert (m0 T4), damit entschieden? |
| F2 | Sprache und Grafik-API: → T1 |
| F3 | Recht bei Veröffentlichung: EULA, Nutzungsrichtlinien, Lizenzen übersetzter Shader. „Minecraft" darf laut Richtlinien nicht Hauptname sein: Wie heißt das Projekt? (R26) |
| F4 | Freisteuerung: welche Teilmenge der Bewegung (Kollision, Schwerkraft, Springen, Schwimmen, Leitern …)? Zwischen Waypoints mit Kollision oder geradlinig? |
| F5 | Pause: Heißt „in einer Anwendung" nur Vollbild, oder auch ein fokussierter Screen an der Wand? |
| F6 | Weltzeit nach einem Weltupdate: Tageszeit aus dem Spiel oder die angehaltene Zeit der Shell? |
| F7 | Wasser- und Blätterbewegung: Vanilla bewegt beides nicht, nur Shader-Packs. Gehört sie zum eingebauten Shader? |
| F8 | Welche Minecraft-Version spielt der Nutzer, welche werden unterstützt? Gelten Resource Packs? Werden Nether und End dargestellt? |
| F9 | Sind echte Kisten im Haus mit Ordnern verbunden, oder gibt es Kisten nur als Symbol (Hotbar, Explorer)? |
| F10 | Zählen Glas, Türen, Slabs und Treppen als Wandfläche oder als Grenze? Was passiert mit einem Screen, dessen Wand nach einem Weltupdate fehlt? |
| F11 | Welche Shell-Grundfunktionen gehören dazu, in welcher Form: Sperrbildschirm, Abmelden/Herunterfahren, Tastaturlayout, Lautstärke und Helligkeit, Netzwerk und Bluetooth, System-Tray, Bildschirmfreigabe, Screenshots? |
| F12 | Eigener Renderer: CLAUDE.md führt ihn als Leitplanke, die alte Featureliste als „Empfehlung, noch nicht endgültig bestätigt". Ist er endgültig entschieden? |
