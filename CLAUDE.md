# Minecraft Shell

Eine Desktop-Shell, die wie eine begehbare Minecraft-Welt aussieht. Sie liest echte Minecraft-Weltdateien mit einem eigenen Renderer und zeigt Programme als Screens an den Wänden eines Hauses. **Status: Brainstorming abgeschlossen, noch nichts implementiert.**

## Dokumente

- [featurelist.md](featurelist.md): Was die Features sind und tun (Teil 1), grober Entwicklungsplan mit Meilensteinen (Teil 2), Entscheidungen und offene Fragen (Teil 3). **Zuerst lesen**, inklusive der Regeln oben in der Datei.
- [designplan.md](designplan.md): Wie der Nutzer mit den Features interagiert (Tastenbelegung, Mausaktionen, Aufbau der Oberfläche) und offene Bedienungsfragen.
- [implementierungsplan.md](implementierungsplan.md): Übersicht der technischen Entscheidungen, die alle Meilensteine betreffen, die Architektur und ein Verzeichnis der Implementierungsdateien.
- [implementierung/](implementierung/): **je eine Datei pro Meilenstein** (`m0-…` bis `m9-…`). Sie ist die alleinige Arbeitsgrundlage der Implementierungs-Session für diesen Meilenstein. Dazu kommen [recherche-offen.md](implementierung/recherche-offen.md) (ausstehende Recherchen) und [recherche-ergebnisse.md](implementierung/recherche-ergebnisse.md) (Funde mit Quellen).
- [linux-umzug.md](linux-umzug.md): Schritt-für-Schritt-Anleitung für den Umstieg des Nutzers auf Linux (Dual Boot, später nur Linux), mit Abhaken.

## Regeln für die Pflege

- **Trennung beachten:** Die Featureliste beschreibt nur *was* ein Feature ist, nie wie man es bedient. Tasten, Klicks und Positionen der Oberfläche gehören in den Designplan. Kommt ein Feature hinzu, steht es in der Featureliste, seine Bedienung im Designplan.
- Entscheidungen des Nutzers zu Features in Teil 3 der Featureliste unter „Entschieden" eintragen, geklärte Fragen aus „Offen" entfernen. Technische Entscheidungen stehen nur im Implementierungsplan. Bedienungsfragen stehen im Designplan (Abschnitt 6).
- Die Meilensteine in Teil 2 der Featureliste sind nur grob. Vor dem Implementieren den Meilenstein in seiner Datei unter `implementierung/` konkretisieren (recherchieren, Entscheidungen klären, Prototypen) und erst dann die Klammer „(grob)" in der Featureliste entfernen.
- **Meilenstein-Dateien** sind für Implementierungs-KIs geschrieben: technisch und knapp. Jede ist für sich allein nutzbar und nennt Ziel, Kern, getroffene Entscheidungen, „Unbedingt beachten", Abnahme und offene Fragen. Hier ist Wiederholung aus anderen Dateien erlaubt. Ändert sich eine übergreifende Entscheidung, alle betroffenen Meilenstein-Dateien mit anpassen.
- **Recherchen:** Eine fertige Recherche kommt mit Quellen in `recherche-ergebnisse.md`. Was für die Umsetzung wichtig ist, wird in die Meilenstein-Datei übernommen, und der Punkt wird aus `recherche-offen.md` gelöscht.
- Nichts erfinden: Was der Nutzer nicht festgelegt hat, kommt als offene Frage in die Dateien, nicht als Entscheidung.
- Dokumente knapp halten. Doppelte Informationen und unnötige Unterüberschriften vermeiden.

## Konventionen

- Sprache der Dokumente und der Zusammenarbeit: **Deutsch** (mit Umlauten).
- Der Nutzer diktiert oft frei per Sprache. Aussagen können abbrechen oder sich widersprechen: Widersprüche und Lücken ansprechen, nicht stillschweigend auflösen.

## Leitplanken des Projekts

- Eigener Renderer, der Minecrafts offene Formate liest (`.mca`, Blockstates und Modelle als JSON, PNG-Texturen). Kein Mojang-Code, keine Mojang-Dateien mitliefern. Texturen kommen aus der lokalen Installation des Nutzers.
- Weltdateien nur lesen, nie schreiben.
- Plattform: **eigenständiges Linux**, auch Minecraft wird dort gespielt. Dual Boot mit Windows ist nur eine Option (vorläufige Alternative: Weltprüfung beim Linux-Start). Sprache und Grafik-API sind noch nicht entschieden.
- Ein eigener Datei-Explorer (Custom Explorer) ist Teil des Projekts: Hotbar, Inventar und Kisten sind echte Ordner.
- **Claude Code programmiert, der Nutzer liest keinen Code** (kann kein Rust). Ergebnisse so erklären und abnehmen, dass sie ohne Codelesen prüfbar sind: sichtbares Verhalten, Screenshots, Messwerte, automatische Tests.
- **Freie Nutzung ohne Bedingungen: Lizenz 0BSD.** Keinen Code aus GPL-Projekten (Waylandcraft, niri, Stardust) übernehmen, nur deren Techniken verstehen und selbst neu umsetzen.
