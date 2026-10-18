# Minecraft Shell

Desktop-Shell als begehbare Minecraft-Welt: Ein eigener Renderer liest echte Weltdateien, Programme hängen als Screens an den Wänden eines Hauses. **Status: M0 läuft (A1–A5 erledigt), Prototypen in `prototypen/`.**

## Dateien

| Datei | Inhalt | Lesen bei |
| ----- | ------ | --------- |
| [features.md](features.md) | was die Features tun, offene Fragen F… | Feature-Fragen; bei Meilensteinen die dort genannten Abschnitte |
| [bedienung.md](bedienung.md) | Tasten, Abläufe, offene Fragen D… | Bedienungsfragen; M1, M3, M4 |
| [implementierungsplan.md](implementierungsplan.md) | übergreifende technische Entscheidungen, Verzeichnis T…, Architektur, Reihenfolge | jedem Meilenstein, technischen Fragen |
| `implementierung/m0-…` bis `m9-…` | je ein Meilenstein: Planungsstand, Kern, T-Details, Technik, Abnahme | Arbeit an diesem Meilenstein |
| [recherche-offen.md](implementierung/recherche-offen.md) | ausstehende Recherchen R… | Recherchen |
| [recherche-ergebnisse.md](implementierung/recherche-ergebnisse.md) | Funde mit Quellen (Archiv) | nur wenn Belege gebraucht werden |

IDs bleiben stabil: F = features.md, D = bedienung.md, T = Verzeichnis im Implementierungsplan, R = Recherchen, P = Prototyp-Tests (m0).

Ausführliche Fassung für den Nutzer: `~/Obsidian/jakobs-obsidian/01_Minecraft Shell/`, eine Datei pro Feature-Gruppe mit Fließtext. Nicht für die Arbeit lesen.

## Pflege

- Jede Information steht an genau einem Ort; anderswo per ID oder Abschnitt verweisen.
- Feature neu oder geändert: *was* → features.md, *wie bedient* → bedienung.md, ausführlich → Obsidian-Datei der Feature-Gruppe. Entscheidungen direkt beim Feature eintragen, die geklärte Frage löschen.
- Technische Entscheidungen: Details in der Meilenstein-Datei, die sie trifft, Status im T-Verzeichnis; übergreifende im Implementierungsplan.
- Als entschieden gilt nur, was der Nutzer bestätigt hat; sonst Empfehlung oder offene Frage. Nichts erfinden.
- Meilenstein vor dem Implementieren konkretisieren: offene Punkte recherchieren, Entscheidungen klären, Teilaufgaben mit Abnahme, Risiken per Prototyp testen. Erst dann im Kopf der Datei Planungsstand „bereit".
- Recherche fertig: Fund mit Quellen → recherche-ergebnisse.md, Umsetzungsrelevantes → Meilenstein-Datei, Punkt aus recherche-offen.md löschen.

## Konventionen

- Deutsch mit Umlauten.
- Der Nutzer diktiert oft per Sprache: Widersprüche und Lücken ansprechen, nicht stillschweigend auflösen.

## Leitplanken

- Eigener Renderer für Minecrafts offene Formate (`.mca`, Blockstates und Modelle als JSON, PNG-Texturen); Status: F12. Kein Mojang-Code, keine Mojang-Dateien mitliefern; Texturen aus der lokalen Installation.
- Weltdateien nur lesen, nie schreiben.
- Plattform: eigenständiges Linux, auch Minecraft läuft dort. Dual Boot mit Windows nur als Übergang.
- **Claude Code programmiert, der Nutzer liest keinen Code** (kann kein Rust). Abnahme ohne Codelesen: sichtbares Verhalten, Screenshots, Messwerte, automatische Tests.
- **Lizenz 0BSD.** Fremden Code nur bei MIT- oder Apache-Lizenz übernehmen; von GPL-Projekten (Waylandcraft, niri, Stardust) und anderem Copyleft nur die Technik.
