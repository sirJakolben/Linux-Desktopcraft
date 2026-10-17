# Minecraft Shell – Featureliste und Entwicklungsplan

Stand: 16.10.2026 · Status: **Brainstorming abgeschlossen, noch nichts implementiert**

> **Hinweis für KI-Sessions (bitte zuerst lesen)**
>
> Diese Datei ist das Ergebnis eines freien Brainstormings mit dem Nutzer (Jakob). Teil 1 beschreibt die Idee und alle bisher genannten Features (nur *was* sie tun). Die **Bedienung** (Tasten, Mausaktionen, Oberfläche) steht im [Designplan](designplan.md). Teil 2 ist ein **grober Orientierungsplan**, kein verbindlicher Entwicklungsplan. Es wurde noch kaum recherchiert, und viele technische Fragen sind offen.
>
> **Regeln für den Umgang mit dem Entwicklungsplan:**
>
> 1. Hinter den Meilensteinen steht „(grob)". Konkretisiere sie zuerst: recherchiere die offenen Punkte, kläre Entscheidungen mit dem Nutzer, zerlege den Meilenstein in Teilaufgaben mit klaren Abnahmekriterien und teste die genannten Risiken mit kleinen Prototypen.
> 2. Erst wenn der Meilenstein konkret genug geplant ist, um ohne Rückfragen loszulegen, entfernst du die Klammer **(grob)** aus der Überschrift und passt die Zeile „Planungsstand" darunter an.

---

# Teil 1: Featureliste

> Diese Liste beschreibt nur, **welche Features es gibt und was sie tun**. Tastenbelegung, Mausaktionen und Aufbau der Oberfläche stehen im [Designplan](designplan.md).

## 1. Vision

- Eine **komplette Desktop-Shell**, die wie eine begehbare Minecraft-Welt aussieht. Beim Start des PCs landet der Nutzer sofort darin.
- Der Nutzer ist nicht wirklich in Minecraft. Die Umgebung wird nur gerendert, als Ersatz für Desktop-Hintergrund, Taskleiste und Fensterverwaltung.
- Die Welt ist ein Nachbau, der **echte Minecraft-Weltdateien** liest. Gleiche Texturen, Blockmodelle und Beleuchtung, aber nur das, was für einen Desktop sichtbar nötig ist.
- Der Rechner soll im Alltag **kaum belastet** werden.
- Die Shell ersetzt den normalen Desktop nicht, sondern **ergänzt** ihn: Man kann wie unter Windows arbeiten (2D-Modus) und jederzeit in die Minecraft-Welt wechseln.
- Der Nutzer wohnt in einem Haus in dieser Welt. An den Wänden hängen Screens mit seinen Programmen, Icons und Ordnern.

## 2. Navigation

- **Waypoints:** Punkte auf dem Boden, zu denen der Spieler automatisch hinläuft, ähnlich wie bei Google Street View. Waypoints lassen sich miteinander verbinden und bilden so ein Wegenetz. Jeder Waypoint hat vier Seiten, jede kann mit einem anderen Waypoint verbunden sein. Läuft der Spieler aus einem Waypoint heraus, geht er zum Waypoint, der mit der Seite in Laufrichtung verbunden ist. Ein neuer Waypoint ist von Anfang an mit dem verbunden, auf dem der Spieler steht. Die Waypoints sind die Standardwege, aber nicht der einzige Weg.
- **Zentrierte Kamera:** Die Standardansicht ist exakt zentriert und fest ausgerichtet. Nach jeder Bewegung kehrt die Kamera dorthin zurück.
- **Freie Kamera:** Der Nutzer kann die Kamera bei Bedarf frei drehen.
- **Drehen um 90°:** Die Ansicht lässt sich in 90°-Schritten drehen.
- **Freisteuerung:** Der Spieler löst sich von den Waypoints und bewegt sich **wie in Minecraft** (eine feste Teilmenge der Spielbewegung, siehe Teil 3). Er kehrt zum Waypoint-Netz zurück, indem er zu einem Waypoint geht.
- **FOV:** Das Sichtfeld ist einstellbar (ran-/rauszoomen).

## 3. Welt und Rendering

- **Minecraft-Look:** Texturen, Blockmodelle, Beleuchtung und Weltstruktur wie im Spiel. Texturen und Modelle kommen zur Laufzeit aus der **lokal installierten** Minecraft-Version und werden nicht mitgeliefert.
- **Minimalistischer Grundzustand:** so wenig wie möglich. Bauen und Abbauen, Spielphysik (außer der Bewegung der Freisteuerung) und alles Unsichtbare entfallen.
- **Behalten:** Sonnenzyklus, Himmel, Wolken, Shader, Wasser- und Blätterbewegung und **animierte Texturen** (dem Nutzer wichtig).
- **Pausieren der Welt:** Ist der Nutzer in einer Anwendung, wird die Welt angehalten: kein Tagesrhythmus, keine Animationen, keine Shader-Berechnung. Im Vollbild entstehen keine neuen Frames. Ist die Welt sichtbar (Screen an der Wand), wird nur neu gezeichnet, wenn sich ein Fenster ändert. So belastet sie den Rechner im Hintergrund kaum. Kehrt der Nutzer zurück, läuft die Zeit **genau dort weiter, wo sie aufgehört hat**.
- **Umfang der Welt:** Die **ganze Welt** ist unterstützt, nicht nur ein Haus. Der Nutzer kann sie frei erkunden. Neue Chunks werden nie erzeugt: Wo die Welt nicht generiert wurde, hört sie auf, und man kann nicht weiterlaufen.
- **Nur Singleplayer-Welten:** Unterstützt werden zunächst nur Singleplayer-Welten, keine Realms und keine Server.
- **Eine Welt zur Zeit:** Es ist immer nur eine Welt geladen, nicht mehrere im Hintergrund.
- **Optionale Mobs:** Standardmäßig gibt es keine Mobs. In den Einstellungen lassen sich Haustiere, alle Mobs mit Namensschild, alle passiven Mobs oder eine Liste einzelner Mobs (zum Beispiel Villager) zulassen. Sie haben nur minimale Logik (zum Beispiel Villager ohne Trades und Workstations, sie laufen zufällig herum).
- **KI-Stufe:** Die Mob-KI ist vereinfacht, die Berechnung lässt sich in den Einstellungen wieder hochstellen.
- **Renderdistanz und Fernsicht:** Größere Renderdistanz als in einem einfachen Viewer. Als Ausbaustufe Fern-Detailstufen im Stil von Distant Horizons. Die Fernsicht zeigt nur bereits geladenes oder besuchtes Gelände und wird einmal beim Weltupdate berechnet und gespeichert.
- **Mod-, Shader- und Welten-Support (Ziel, späte Ausbaustufe):** Support wie in Minecraft. Mods, Shader und modifizierte Welten sollen sich nutzen lassen. Der Weg ist offen.
  - Gewünscht ist ein **eigener, effizienter Renderer** (zum Beispiel in Rust) mit wenig Last.
  - **Mods:** Ein **Adapter** analysiert, was eine Mod verändert, und zieht nur das Wichtige heraus: Custom-Blöcke, Texturen, eventuell Custom-Mobs. Rezepte, Blockverhalten und Texturanimationen der Mods werden weggelassen. Einschätzung (ungeprüft): Blöcke und Texturen aus den Asset-Dateien zu lesen ist machbar. Schwierig bis kaum machbar sind Blöcke mit eigenem Darstellungscode und Mods, die das Rendering verändern.
  - **Shader:** Es gibt einen **eingebauten Shader**, vielleicht mehrere Versionen. Dafür werden bekannte Minecraft-Shader angeschaut, in eigenen Shadercode **übersetzt** und der unnötige Code entfernt. Genau wie bei Mods sind das die unterstützten Shader: Hat der Nutzer einen davon für sein echtes Minecraft installiert, nutzt die Desktop-Version ihn ebenfalls. Direkte Kompatibilität mit beliebigen Shader-Packs (OptiFine/Iris) ist nicht das Ziel.

## 4. Fenster in der Welt

- **Screens:** Programme erscheinen als echte Fenster, als Live-Textur an den Wänden des Hauses. Desktop-Icons hängen als kleine Screens (wie Item-Frames) an der Wand.
- **Persistenz und Fernsicht:** Geöffnete Fenster bleiben an der Wand offen und sind auch aus der Ferne sichtbar. Nahe Fenster sind scharf und bedienbar, entfernte wirken kleiner und perspektivisch verzerrt. Eingaben werden korrekt an das echte Fenster weitergegeben.
- **Zwischen 2D und Welt wechseln:** Ein einzelnes 2D-Fenster lässt sich in die Welt setzen und ein Wand-Screen wieder zum 2D-Fenster machen. Das ist der **einzige Weg** zwischen beiden Ansichten. Mini-Desktops wirken nicht auf die Welt.
- **Vollbild:** Ein Fenster lässt sich im Vollbild öffnen. Dort wird die Welt nicht gerendert.
- **Größe ändern:** Ein Screen lässt sich über mehrere Blöcke vergrößern, aber nur rechteckig (keine L-Formen). Die Skalierung ist **intelligent**: Die **Wand ist die Begrenzung**. Der Screen endet dort, wo die Wand endet, und lässt sich nicht darüber hinausziehen. **Blöcke ohne Hitbox** werden ignoriert und lassen sich überziehen: Blumen, Gras, Fackeln, Töpfe und alles andere Kleine, Durchlaufbare, das kein voller Block, keine Slab und keine Treppe ist.
- **Drei Ansichten eines offenen Fensters:** Ein laufendes Fenster hat drei Darstellungsweisen, zwischen denen man per Drag & Drop wechseln kann:
  1. **Icon-Ansicht:** ein kleines Icon mit Enchantment-Glow, das zeigt, dass dieses Fenster offen ist. Es liegt in der Hotbar oder im Inventar, nicht in Kisten.
  2. **Fenster-Ansicht:** das Fenster als normales 2D-Fenster auf dem Bildschirm (Desktop), wie unter einem herkömmlichen Betriebssystem, auch im Vollbild.
  3. **Wand-Ansicht:** das Fenster als Screen an einer Wand der Minecraft-Welt.
- **Ansichten sind unabhängig:** Dasselbe Fenster kann gleichzeitig in mehreren Ansichten existieren (zum Beispiel auf dem Desktop und in der Welt) oder nur in einer. Keine Ansicht ist Pflicht: Ein Fenster auf dem Desktop braucht kein Icon in der Hotbar.
- **2D-Modus und Minecraft-Modus:** Die Shell ergänzt die 3D-Minecraft-Nutzung um die **normale Desktop-Nutzung**. Im **2D-Modus** liegen mehrere Fenster nebeneinander auf einem Bildschirm, genau wie unter Windows. Der Nutzer wechselt zwischen 2D-Modus und Minecraft-Modus.
- **Mini-Desktops:** Beim Wechsel in den Minecraft-Modus schrumpfen die offenen 2D-Fenster in einer kurzen Animation zu einem **Mini-Desktop**: einem kleinen Rechteck unten links, das die Fenster verkleinert zeigt und auch beim Laufen durch die Welt sichtbar bleibt. Ein Klick darauf lässt die Fenster wieder auf den ganzen Bildschirm aufgehen und aktiviert den 2D-Modus. Es gibt **zwei Mini-Desktops** (unten links und unten rechts).
  - Ein Mini-Desktop entsteht **erst, wenn mindestens zwei offene Fenster** auf demselben Bildschirm liegen. Ein einzelnes Fenster wird beim Wechsel direkt zum **Icon in der Hotbar**.
  - Ist ein Fenster **identisch** mit einem, das schon in einem Mini-Desktop liegt (dieselbe Anwendung und dasselbe Dokument/Projekt), gehört es in diesen Mini-Desktop. Ein anderes Projekt in derselben Anwendung gilt als anderes Fenster.
  - Die **übrigen** Fenster kommen in den nächsten freien Mini-Desktop, wenn es mindestens zwei sind. Bleibt nur eins übrig, wird es zum Icon in der Hotbar (falls es noch kein Icon dafür gibt).
  - Ist kein Mini-Desktop frei, werden die Fenster direkt zu Icons.
  - **Schließen:** Ein Mini-Desktop lässt sich schließen. Dabei wird nur der Mini-Desktop gelöscht, die **Fenster bleiben offen**.
- **Mehrere Wand-Screens:** Dasselbe Fenster darf in der Welt **beliebig oft** an Wänden hängen. Der **zuletzt benutzte** Wand-Screen gilt als **Prime-Fenster**. Zu ihm kann man sich **teleportieren**; dabei geht es zur Position, an der das Fenster in der Welt geöffnet wurde.
- **Minimieren:** Ein Fenster kann als Icon in die Hotbar gelegt und von dort wieder auf eine Wand oder den Desktop gesetzt werden (Umsetzen).
- **Schließen und Verknüpfung:** Wird ein offenes Fenster geschlossen, werden alle seine Wand-Screens zu **Verknüpfungen zur Anwendung**. Statt des Programm-Icons zeigen sie den **letzten Screenshot der Sitzung** als Bild. Ein Klick darauf startet die Anwendung wieder, und der Screen wird aktualisiert (zum Beispiel ein Word-Dokument mit dem letzten Stand als Vorschau).
- **Platzieren:** Beim Platzieren zeigt eine Vorschau den Zielblock, ein neuer Screen belegt zunächst einen Block (1×1).

## 5. Shell-Elemente

Einordnung: **Hotbar = Taskleiste, Inventar = Desktop, Kiste = Ordner, Nether-Portal = Weltenwechsel.**

- **Tief ins Dateisystem integriert:** Hotbar, Inventar und Kisten sind **echte Ordner** im Dateisystem und im Datei-Explorer sichtbar. Sie sind keine Kopien oder Verknüpfungslisten, sondern die Ordner selbst. Bewegt der Nutzer ein Item zwischen ihnen, wird die Datei oder der Ordner **physisch verschoben**. So ordnet er sein Dateisystem direkt in der Welt.
- **Hotbar:** Ersatz für die Taskleiste, im Look der Minecraft-Hotbar. Sie ist ein eigener Ordner und **immer sichtbar**. Zeigt offene und minimierte Programme. Standardmäßig 9 Slots, mit mehr Platz bei Bedarf. Anwendungen daraus lassen sich in der Welt als Screen platzieren. Offene Anwendungen siehe unten.
- **Übersicht offener Anwendungen:** Ein eigenes Symbol neben Kisten- und Portal-Symbol zeigt eine **Liste aller offenen Anwendungen** in einem Fenster. Von dort lässt sich jede Anwendung öffnen oder in eine der drei Ansichten ziehen (Icon in die Hotbar, Fenster auf den Desktop, Screen in die Welt).
- **Offene Anwendungen:** Laufende Anwendungen erscheinen als **Items mit Enchantment-Glow** aus Minecraft (Icon-Ansicht, siehe Abschnitt 4). Ein Klick darauf öffnet das Fenster.
  - Sie liegen in einer **eigenen Sektion** im Hotbar- und Inventar-Ordner und sehen aus wie Dateien. Echte Dateien sind sie nicht: Die Sektion gibt es nur im eigenen Explorer, andere Programme sehen sie nicht. Sie bleiben **nur in Hotbar und Inventar** und lassen sich nicht in Kisten verschieben, dürfen aber dort frei umhergeschoben werden.
  - Am Icon gibt es oben rechts ein **X**, das die Anwendung schließt.
  - Beim Hovern erscheint ein **Mini-Tooltip als Live-Vorschau**, die zeigt, was in der Anwendung gerade passiert.
  - **Jede Instanz ist ein eigenes Item.** Zwei offene Word-Fenster sind zwei Items, nicht eins mit zwei Instanzen.
- **Suche:** Sucht auf dem PC.
- **PC-Informationen und Schnellzugriff:** In einer Ecke des Bildschirms stehen Infos wie Suchleiste, Ausschalten-Knopf und Uhrzeit. Eine **anpassbare Symbolleiste** bietet Schnellzugriff auf Weltenwechsel, Datei-Explorer, offene Anwendungen und weitere Werkzeuge (zum Beispiel ein Snipping-Tool).
- **Standardoberfläche im 2D-Modus:** Nur die Hotbar (Taskleiste) ist sichtbar; die Symbolleiste erscheint nur bei Bedarf.
- **Inventar:** Ersatz für den Desktop mit allen oft gebrauchten Icons und Verknüpfungen, in jeder App erreichbar, als schneller Starter. Das Inventar ist ebenfalls ein eigener Ordner.
- **Kisten:** Jeder Ordner im Dateisystem ist eine Kiste. Im eigenen Explorer erscheint **überall ein Kistensymbol statt des Ordnersymbols**. Inhalt lässt sich frei zwischen Kisten, Inventar und Hotbar verschieben (physisch). Innerhalb einer Kiste lassen sich Items **frei platzieren wie in Minecraft**, statt nur nach Name sortiert zu sein. Die Position wird gespeichert.
- **Eigener Datei-Explorer:** Der Datei-Explorer wird **komplett neu geschrieben**, als Custom Explorer, der extra für das Minecraft-System ausgelegt ist (Kisten, freie Item-Positionen, Hotbar und Inventar als Ordner). Er ist der Ort für weitere Ideen zu Aussehen, Verhalten und Optionen der Oberfläche; diese kommen später dazu.
- **Nether-Portal:** Ersetzt die Ender Chest. Steht für den Weltenordner mit allen Welten und wechselt die Welt, zur letzten Position in dieser Welt (Fallback: Spawnpunkt). Der Wechsel wird mit einer Überblendung kaschiert.
- **Welt in Minecraft starten:** Neben jeder Welt in der Weltenauswahl startet ein eigener Knopf diese Welt im echten Minecraft, wie ein Launcher. Gespielt wird unter Linux in der Shell. Beim Beenden folgt das automatische Weltupdate (Abschnitt 6).
- **Einstellungen:** Zentrale Einstellungen, unter anderem für Mobs und KI-Stufe.

## 6. Weltsynchronisierung

- Jede bestehende Minecraft-Welt (Java Edition) lässt sich **ohne manuelle Konvertierung** laden. Es ist **dieselbe Welt** wie im Spiel, nicht nur eine ähnliche.
- **Automatisches Update:** Beim Beenden von Minecraft erkennt der Desktop das Prozessende (das Spiel hat dann vollständig gespeichert) und kopiert die Welt inkrementell, nur geänderte Region-Dateien, in einen eigenen Cache. Danach werden nur geänderte Chunks neu aufgebaut. Die Originalwelt wird nie verändert.
- **Vorläufige Alternative für Dual Boot:** Wird Minecraft unter Windows gespielt, läuft die Shell währenddessen nicht. Dann prüft sie die Welten auf der Windows-Partition beim Linux-Start und aktualisiert geänderte. Das ist nur eine Übergangslösung; der eigentliche Weg ist Minecraft unter Linux.
- **Fortschrittsanzeige:** Ein Banner zeigt den Fortschritt und am Ende einen Haken.
- **Umfang:** Nur Singleplayer-Welten. **Realms und Server sind gestrichen** (zunächst).
- **Eigene Daten:** Fensterlayout, Item-Positionen in Kisten und Waypoints werden **getrennt von der Welt** gespeichert, zum Beispiel in einer Layout-Datei pro Welt.

## 7. Performance

- **Nur zeichnen, wenn nötig.** Ist der Nutzer ausgetabt oder im Vollbild, wird die Welt nicht gerendert. Mobs sind standardmäßig aus, die KI ist vereinfacht.
- Fertige Meshes werden gespeichert, damit Welten von der SSD schnell laden (Ziel: wenige Sekunden). Chunks werden von innen nach außen nachgeladen, mit Nebel als Kaschierung.
- Es ist immer nur die aktive Welt geladen, andere gibt es nur als Vorschaubild. Fenster außerhalb des Blickfelds werden gedrosselt (ca. 1 Bild pro Sekunde).
- Grobe Schätzungen (ungemessen): 100–300 MB VRAM bei ca. 12 Chunks Renderdistanz, Fernsicht zusätzlich 1–3 GB. Hardware des Nutzers: Nvidia GeForce RTX **5060 Ti** mit **16 GB VRAM**.

## 8. System und Shell

- Vollständige Desktop-Shell: Der PC startet direkt in der Welt.
- Plattform: **Linux, eigenständig.** Die Shell ist dafür ausgelegt, allein auf Linux zu laufen, auch Minecraft wird dort gespielt. Dual Boot mit Windows ist nur eine Option. Linux gibt mehr Kontrolle (eigener Window-Manager/Compositor). Die Windows-Taste heißt dort Super-Taste; in dieser Dokumentation steht weiter „Win".
- Später nötig: Benachrichtigungen, Audio, mehrere Monitore, Energiesparen und ein **Notfallmodus** (Rückfall auf normalen Desktop bei Absturz).

## 9. Leitplanken und Recht

- Ursprünglicher Wunsch: Minecraft-Code verwenden. Hinweis: Der Kauf des Spiels ist nur eine Spiellizenz und gibt nicht automatisch das Recht, Mojang-Code in ein anderes Produkt zu übernehmen. Keine Rechtsberatung, EULA und Nutzungsrichtlinien prüfen.
- Empfohlen: **eigener Renderer**, der Minecrafts offene Formate liest (`.mca`, Blockstates/Modelle als JSON, PNG-Texturen aus der lokalen Installation). Keine Mojang-Dateien mitliefern.
- Nur lesen, nie in die Weltdateien schreiben. Der Parser muss die Versionen kennen, die der Nutzer spielt (Version steht in `level.dat`).
- Verworfene Alternative: eine Mod im laufenden Minecraft (das ganze Spiel müsste dauerhaft im Hintergrund laufen).
- Beim Übersetzen bekannter Shader in eigenen Code die **Lizenzen der Shader-Autoren** beachten (Code ist urheberrechtlich geschützt, Lizenz je Shader prüfen).

## 10. Späte Ausbaustufe: Multiplayer-Sessions

- Nach dem Release („next level"): Mehrere Nutzer sind in derselben Minecraft-Welt. Man kann zu anderen Leuten hinlaufen und die Fenster ansehen und öffnen, die sie oft geöffnet haben.
- Setzt Netzwerk, Synchronisierung und Rechtekonzept voraus (wer sieht und bedient welche Fenster). Noch nicht ausgearbeitet.

---

# Teil 2: Grober Entwicklungsplan

> **Wichtig:** Nur zur **Orientierung** (Reihenfolge, Risiken, Größenordnung). **Keine Grundlage zum direkten Losprogrammieren.** Zeitangaben sind sehr grobe Schätzungen für eine Person, die nebenbei arbeitet. Jede spätere Session konkretisiert den Meilenstein vor der Implementierung in seiner eigenen Datei unter [implementierung/](implementierung/) (Verzeichnis im [Implementierungsplan](implementierungsplan.md)) und entfernt dann hier die Klammer **(grob)**. Technische Entscheidungen stehen ebenfalls dort.
>
> Risikostufen: niedrig / mittel / hoch / sehr hoch.

## Meilenstein 0: Entscheidungen und Mini-Tests (grob)

- **Zeit:** 1–3 Wochen · **Risiko:** hoch
- Plattform ist entschieden (Linux, Dual Boot). Sprache und Grafik-API festlegen (zum Beispiel Rust + wgpu, C++ + Vulkan oder Godot). Wegwerf-Prototypen für die riskantesten Punkte, vor allem: ein echtes Fenster als Live-Textur auf einer 3D-Fläche, mit klickbarer Maus. Trägt dieser Prototyp nicht, kippt das Projekt.
- **Planungsstand:** Nicht bereit. Es fehlen Sprache und Grafik-API sowie eine Prototypenliste mit Erfolgskriterien.

## Meilenstein 1: Weltviewer (grob)

- **Zeit:** 3–6 Wochen · **Risiko:** mittel
- Region-Dateien lesen, Chunks zu Meshes umrechnen, Texturen und Modelle laden, Waypoint-Bewegung (Win + WASD), Kamera (zentriert, Mittelmaustaste, Randpfeile), Freisteuerung.
- Zeitfresser: Sonderformen (Treppen, Zäune, Türen, Mehrteilblöcke über Blockstates), Biomfarben, Weltformat-Unterschiede.
- **Planungsstand:** Nicht bereit. Es fehlen Zielversionen, eine Liste der Blocktypen und eine Datenstruktur für Chunks/Meshes.

## Meilenstein 2: Minecraft-Look (grob)

- **Zeit:** 3–6 Wochen · **Risiko:** mittel
- Smooth Lighting, Ambient Occlusion, Sonnenzyklus, Himmel, Wolken, animierte Texturen, Wasser- und Blätterbewegung, Transparenz, Nebel, Pausieren beim Austabben.
- Schwierig: Transparenz in richtiger Reihenfolge; „sieht genau aus wie Minecraft" ist Feintuning ohne klares Ende.
- **Planungsstand:** Nicht bereit. Es fehlt die erlaubte Abweichung vom Original.

## Meilenstein 3: Fenster in der Welt (grob)

- **Zeit:** 6–12+ Wochen · **Risiko:** sehr hoch (größter Zeitfresser)
- Fenster als Wand-Screens, Klicks per Raycast in Fensterkoordinaten, Größe per Eckpunkten, Minus/Kreuz, Vollbild per Doppelklick.
- Probleme: Pop-ups, Tooltips und Dialoge sind eigene Fenster; Tastatur, Fokus, Zwischenablage, Drag & Drop, IME; HiDPI und Größenänderung; Videos und Spiele (Latenz, kopiergeschützte Inhalte werden oft schwarz). Unter Linux (Wayland) muss der Compositor selbst geschrieben werden (zum Beispiel mit wlroots oder Smithay). Dafür gibt es volle Kontrolle über Fenster und Eingaben. Windows-Programme laufen dort nur über Wine/Proton oder im Dual Boot.
- **Planungsstand:** Nicht bereit. Abhängig von Plattform und Meilenstein 0; es fehlt eine Recherche zu Capture-Technik und Eingabeweitergabe.

## Meilenstein 4: Shell-Bedienung (grob)

- **Zeit:** 3–6 Wochen (plus eigener Datei-Explorer, noch nicht geschätzt) · **Risiko:** mittel
- Hotbar mit Suchleiste, Kisten-, Einstellungs- und Portal-Symbol; Drag in die Welt; Ordner-Integration (Hotbar, Inventar, Kisten als echte Ordner, Kistensymbol im Explorer, freie Item-Positionen); Nether-Portal; Inventar (Win + E); Waypoints (setzen, verbinden); Einstellungen (Mobs, KI-Stufe); Layout pro Welt speichern.
- Eigener Datei-Explorer (Kisten-Raster, freie Item-Positionen, Hotbar und Inventar als Ordner) ist ein eigenes Teilprojekt.
- Die Tastenbelegung mit der Super/Win-Taste ist unter Linux unkritisch, da die Shell selbst die Eingaben steuert (Konflikte mit Systemkürzeln entfallen).
- **Planungsstand:** Nicht bereit. Abhängig von Plattform und Fenstertechnik; es fehlen Detaildesigns für Hotbar, Inventar und Layout-Speicherung.

## Meilenstein 5: Weltsynchronisierung (grob)

- **Zeit:** 2–4 Wochen · **Risiko:** niedrig bis mittel
- Prozessende von Minecraft erkennen, Welt inkrementell kopieren, Banner mit Haken, letzte Spielerposition aus `level.dat`. Welt aus der Weltenauswahl in Minecraft starten (Launcher-Funktion). Vorläufige Alternative: Prüfung der Windows-Partition beim Linux-Start. Realms und Server sind gestrichen, nur Singleplayer-Welten.
- **Planungsstand:** Teilweise geplant (Ablauf klar), aber nicht bereit. Offen sind die Weltversionen und der Weg, Minecraft zu starten (eigener Launcher oder vorhandenen Launcher steuern).

## Meilenstein 6: Performance und Fernsicht (grob)

- **Zeit:** 4–10 Wochen · **Risiko:** hoch bei der Fernsicht
- Meshes zwischenspeichern, Chunks nachladen, Rendern nur bei Bedarf, Fenster drosseln, Distant-Horizons-Fernsicht. Kann ganz ans Ende geschoben werden.
- **Planungsstand:** Nicht bereit. Es fehlen das Renderdistanz-Ziel und Messungen (VRAM: 16 GB).

## Meilenstein 7: Mod- und Shader-Support (grob)

- **Zeit:** nicht abschätzbar · **Risiko:** sehr hoch
- Mod-Adapter (Blöcke, Texturen, eventuell Mobs) und eingebaute Shader (bekannte Shader übersetzt und verschlankt), siehe Abschnitt 3. Wahrscheinlich eine der letzten Stufen.
- **Planungsstand:** Nicht bereit. Es braucht erst Recherche zu Mod-Formaten, Modloadern, der Auswahl der Shader und deren Lizenzen.

## Meilenstein 8: Echte Shell (grob)

- **Zeit:** 3–8 Wochen · **Risiko:** hoch
- Start nach dem Login, Benachrichtigungen, Audio, mehrere Monitore, Energiesparen, Absturzschutz. Der Notfallmodus muss existieren, **bevor** die Shell Standard wird.
- **Planungsstand:** Nicht bereit. Abhängig von allen vorigen Meilensteinen und der Plattform.

## Meilenstein 9: Multiplayer-Sessions (grob, nach dem Release)

- **Zeit:** nicht abschätzbar · **Risiko:** sehr hoch
- Gemeinsame Welt mehrerer Nutzer, Fenster anderer ansehen und öffnen, siehe Abschnitt 10.
- **Planungsstand:** Nicht bereit. Bewusst der letzte Schritt nach dem Release.

## Reihenfolge und größte Unsicherheiten

1. Meilenstein 0 (vor allem der Fenster-Prototyp).
2. Meilenstein 1 und 2 zunächst als normale Anwendung unter Linux (noch nicht als Shell), damit früh Ergebnisse sichtbar sind.
3. Meilenstein 3 im selben Rahmen. Dann 4 und 5.
4. Meilenstein 6, 7 und 8 zuletzt. Meilenstein 9 erst nach dem Release.

Größte Unsicherheiten: **Fenster als bedienbare 3D-Flächen** (Abbruchrisiko), **eigener Compositor und eigener Datei-Explorer** und **Fernsicht, Look-Parität sowie Mod-/Shader-Support** (nicht riskant für das Projekt, aber leicht endlos).

---

# Teil 3: Entscheidungen und offene Fragen

Bei jeder Session pflegen: Entscheidungen oben eintragen, Antworten des Nutzers kurz notieren.

## Entschieden

- **Eigener Renderer**, der Minecrafts Dateiformate liest. Kein Mojang-Code (Empfehlung, noch nicht endgültig bestätigt).
- Weltupdate-Auslöser ist das **Beenden von Minecraft**, mit Banner und Haken.
- Das **Nether-Portal** (statt Ender Chest) wechselt die Welt, zur letzten Position.
- Bewegung über **Waypoints** plus **Freisteuerung**; die Kamera ist standardmäßig zentriert (Bedienung: siehe [Designplan](designplan.md)).
- Screens lassen sich nur rechteckig vergrößern.
- Die Welt **pausiert** beim Austabben und läuft danach **exakt dort weiter**, wo sie aufgehört hat. Das Mitschwenken der Kamera bei gehaltener Win-Taste entfällt.
- Skalierung von Screens: **Wand ist die Grenze**, Blöcke ohne Hitbox (Blumen, Gras, Fackeln, Töpfe) werden ignoriert.
- **Nur Singleplayer-Welten**, **eine Welt zur Zeit**, **keine Realms und keine Server** (zunächst). Die **ganze Welt** ist erkundbar, aber es werden keine neuen Chunks erzeugt.
- **Shader:** eingebauter, eigener Shader (übersetzte und verschlankte Versionen bekannter Shader); wirkt wie bei Mods als Liste unterstützter Shader.
- **Multiplayer-Sessions** sind eine späte Ausbaustufe nach dem Release.
- Grafikkarte: RTX **5060 Ti**.
- Fragen zur Bedienung (Tasten, Mausaktionen, Oberfläche) stehen im Designplan.
- Mobs standardmäßig aus, in den Einstellungen zuschaltbar, nur mit minimaler Logik.
- Plattform: **eigenständiges Linux**, Minecraft wird unter Linux in der Shell gespielt. Der Nutzer stellt seinen ganzen PC auf Linux um. Dual Boot ist nur eine Option bzw. Übergang; dafür gibt es vorläufig die Prüfung der Windows-Partition beim Linux-Start.
- **Welt in Minecraft starten:** eigener Knopf neben jeder Welt in der Weltenauswahl (Launcher-Funktion).
- **Pausieren:** Zeit, Animationen und Shader halten an. Ist die Welt sichtbar, wird nur bei Fensteränderungen neu gezeichnet.
- **Offene Anwendungen** erscheinen nur im eigenen Explorer, nicht als echte Dateien.
- **Freisteuerung:** Es wird eine feste Teilmenge der Minecraft-Bewegung nachgebaut, nicht die gesamte Spielphysik.
- Eigener, neu geschriebener **Datei-Explorer** (Custom Explorer) für das Minecraft-System; das Kistensymbol gilt dort überall.
- Offene Anwendungen sind Items mit **Enchantment-Glow**, eigene Sektion im Hotbar-/Inventar-Ordner, jede Instanz ein eigenes Item, nicht in Kisten verschiebbar.
- **2D-Modus** wie ein normaler Desktop (mehrere Fenster auf einem Bildschirm) ergänzt die Minecraft-Welt; der 2D-Modus ist ein eigener Modus, keine Ebene über der Welt. Beim Wechsel in die Welt schrumpfen die Fenster in einen von **zwei Mini-Desktops** (unten links, unten rechts), die in der Welt sichtbar bleiben; Ein Mini-Desktop entsteht erst bei mindestens zwei Fenstern, einzelne Fenster werden zu Icons in der Hotbar; Fenster ohne Platz ebenso.
- Mini-Desktops wirken **nicht** auf die Welt. Zwischen 2D-Fenster und Wand-Screen wechselt man nur einzeln (in die Welt ziehen, zurück per Doppelklick).
- Im 2D-Modus ist standardmäßig nur die Hotbar sichtbar, dazu eventuell PC-Infos in der Ecke; eine anpassbare Symbolleiste erscheint bei gehaltener Win-Taste.
- „Teleport to" ist **ausgegraut**, solange es noch keinen Wand-Screen des Fensters gibt.
- **Drei unabhängige Ansichten** eines offenen Fensters (Icon, Desktop-Fenster, Wand-Screen), beliebig kombinierbar; beliebig viele Wand-Screens pro Fenster; der zuletzt benutzte ist das **Prime-Fenster** mit Teleport. Beim Schließen werden Wand-Screens zu **Verknüpfungen mit letztem Screenshot**. Dazu ein Symbol mit Liste aller offenen Anwendungen.
- Hotbar, Inventar und Kisten sind **echte Ordner** im Dateisystem (Verschieben ist physisch). Jeder Ordner erscheint als Kiste.
- Layout, Item-Positionen in Kisten und Waypoints werden **getrennt von der Weltdatei** gespeichert. Animierte Texturen sind gewünscht.

## Offen

1. **Linux-Details:** Welche Distribution? Wie kommen Windows-Programme in die Welt (nur Linux-Apps, oder Wine/Proton)? Mit welchem Launcher spielt der Nutzer unter Linux (offizieller Launcher, Prism Launcher …)?
2. **Sprache und Grafik-API/Engine:** noch nicht gewählt (Rust-Renderer ist die Idee). Wird im [Implementierungsplan](implementierungsplan.md) geklärt.
3. **Rechtliches:** EULA und Nutzungsrichtlinien von Mojang sowie Lizenzen der übersetzten Shader prüfen, falls das Projekt später veröffentlicht wird. Laut Nutzungsrichtlinien darf „Minecraft" nicht der Hauptname sein: Wie soll das Projekt bei einer Veröffentlichung heißen? (Ergebnisse siehe [recherche-ergebnisse.md](implementierung/recherche-ergebnisse.md), Abschnitt R26.)
4. **Freisteuerung und Physik:** Welche Teilmenge der Minecraft-Bewegung (Kollision, Schwerkraft, Springen, Schwimmen, Leitern …)? Läuft der Spieler zwischen Waypoints mit Kollision oder in gerader Linie?
5. **Pausieren:** Was heißt „in einer Anwendung"? Nur Vollbild, oder auch ein Screen an der Wand mit Fokus?
6. **Weltzeit nach einem Weltupdate:** Tageszeit aus dem Spiel übernehmen oder die angehaltene Zeit der Shell behalten?
7. **Wasser- und Blätterbewegung, „Shader" unter Behalten:** Vanilla-Minecraft bewegt beides nicht, das machen nur Shader-Packs. Gehört das zum eingebauten Shader?
8. **Versionen und Umfang der Welt:** Welche Minecraft-Version spielt der Nutzer, welche müssen unterstützt werden? Sollen Resource Packs gelten? Werden Nether und End dargestellt?
9. **Kisten in der Welt:** Sind echte Kisten im Haus mit Ordnern verbunden, oder gibt es Kisten nur als Symbol (Hotbar, Explorer)?
10. **Wand für Screens:** Zählen Glas, Türen, Slabs und Treppen als Wandfläche oder als Grenze? Was passiert mit einem Screen, dessen Wand nach einem Weltupdate fehlt?
11. **Fehlende Shell-Grundfunktionen:** Sperrbildschirm, Abmelden/Herunterfahren, Tastaturlayout, Lautstärke und Helligkeit, Netzwerk und Bluetooth, System-Tray, Bildschirmfreigabe, Screenshots. Welche gehören dazu, und in welcher Form?
