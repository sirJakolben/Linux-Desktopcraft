# Minecraft Shell – Implementierungsplan (Übersicht)

Stand: 16.10.2026 · Status: **In Planung, noch nichts implementiert**

> **Hinweis für KI-Sessions (bitte nach der [Featureliste](featurelist.md) lesen)**
>
> Diese Datei ist die Übersicht. Sie enthält die **technischen Entscheidungen, die alle Meilensteine betreffen**, die vorgeschlagene Architektur und ein Verzeichnis der Implementierungsdateien.
>
> - **Für jeden Meilenstein gibt es eine eigene Datei** in [implementierung/](implementierung/). Sie ist die alleinige Arbeitsgrundlage der Implementierungs-Session für diesen Meilenstein. Was dort fehlt, wird dort ergänzt, nicht hier.
> - Als „entschieden" steht nur, was der Nutzer bestätigt hat. Empfehlungen ohne Bestätigung stehen unter „Offen".
> - Ändert sich eine Entscheidung hier, werden die betroffenen Meilenstein-Dateien mit angepasst.
> - Entscheidungen zu den Features selbst stehen in Teil 3 der Featureliste, die Bedienung im [Designplan](designplan.md).

---

## 1. Technische Entscheidungen

### Entschieden

- **Dokumentaufteilung:** Diese Übersicht plus je eine Datei pro Meilenstein und je eine Datei für offene und abgeschlossene Recherchen (siehe Abschnitt 3). Teil 2 der Featureliste bleibt der grobe Gesamtplan.
- **T5 Entwicklungsweg:** Alles wird **direkt unter Linux** entwickelt. Unter Windows lauffähig zu sein ist kein Ziel. Der Nutzer stellt seinen PC komplett auf Linux um; Windows bleibt nur übergangsweise im Dual Boot (für die laufende Mod-Entwicklung „Smart Villagers").
- **Programmierung durch Claude Code:** Der Nutzer programmiert nicht selbst und kann kein Rust. Daraus folgt:
  - Abnahmekriterien müssen ohne Codelesen prüfbar sein: sichtbares Verhalten, Screenshots, Messwerte.
  - Automatische Tests und `cargo clippy` gehören zu jedem Schritt.
  - `unsafe`-Code bleibt auf wenige kleine Module beschränkt.
- **T17 Lizenz 0BSD:** Jeder darf alles benutzen, verändern und weitergeben. „Keine Lizenz" hieße rechtlich „alle Rechte vorbehalten"; 0BSD ist OSI-anerkannt, ganz ohne Bedingungen und wirkt auch in Deutschland.
  - Die `LICENSE`-Datei kommt mit dem ersten Code ins Repository.
  - Code aus GPL-Projekten (Waylandcraft, niri, Stardust) wird **nicht übernommen**; sie dienen nur als Anschauung für Techniken.
  - Abhängigkeiten unter MIT oder Apache (Smithay, wgpu) sind in Ordnung.
- **T18 Installationsort:** Linux kommt in **500 GB auf der NVMe-SSD** (C: wird verkleinert), mit **eigener EFI-Partition** (die von Windows hat nur 100 MB). So lässt sich Windows später ohne Umbau löschen. Details in [linux-umzug.md](linux-umzug.md).
- **Hardware (ausgelesen 16.10.2026):**
  - RTX 5060 Ti mit **16 GB VRAM**
  - NVMe-SSD WD_BLACK SN850X, 2 TB
  - HDD Toshiba, 1 TB (D:, fast leer)

### Offen

| Nr. | Entscheidung | Empfehlung | Meilenstein |
| --- | ------------ | ---------- | ----------- |
| T1 | Sprache und Grafik-API | **Rust + wgpu** (≥ 30), rohes Vulkan über wgpu-hal nur für dmabuf und Sync; Rückfall: Smithays GlesRenderer | [M0](implementierung/m0-entscheidungen-und-prototypen.md) |
| T2 | Compositor-Bibliothek | **Smithay** (Git-Master auf einen geprüften Commit festgelegt); Rückfall: wlroots über FFI | M0 |
| T3 | Prozessaufteilung | **ein Prozess**; die Welt als eigene Crate hinter einer Schnittstelle, später auslagerbar. Der Notfallmodus läuft zunächst über eine zweite Sitzung (Plasma) | M0, M8 |
| T4 | Linux-Distribution | **Fedora KDE Plasma**, NVIDIA 595 (offen) über RPM Fusion, Snapper. Die Umzugsanleitung geht bereits davon aus | M0 |
| T6 | Unterstützte Minecraft-Versionen, Wahl der Asset-Version | – | [M1](implementierung/m1-weltviewer.md) |
| T7 | Quelle für Blockdaten, die nicht in den Assets stehen | – | M1 |
| T8 | Render-Pipeline (forward oder deferred) | – | [M2](implementierung/m2-minecraft-look.md) |
| T9 | Datenstruktur für Chunks und Meshes, Mesh-Cache | – | M1 |
| T10 | Bildtaktung je Zustand | Leerlauf 0 Bilder, stille Kamera 20 Hz, Bewegung volle Rate, Vollbild Direct Scanout | M0, M2 |
| T11 | Fensterauflösung und Größenaushandlung | logische Größe und Skala unabhängig von der Blockgröße, Pixeldichte pro Block einstellbar | M0, [M3](implementierung/m3-fenster-in-der-welt.md) |
| T12 | Verankerung der Screens, Layout-Datei pro Welt | – | M1, M3 |
| T13 | Bewegungsmodell (Teilmenge der Minecraft-Physik) | – | M1 |
| T14 | Weltcache und Erkennung von Änderungen | – | M1, [M5](implementierung/m5-weltsynchronisierung.md) |
| T15 | Format der Einstellungen | – | M1 |
| T16 | Minecraft starten: vorhandenen Launcher steuern oder eigener Launcher | – | M5 |
| T19 | Sprache im Code (Bezeichner, Kommentare) | – | M0 |

## 2. Architektur (Vorschlag, gilt mit T1–T3)

Ein Cargo-Workspace:

| Crate        | Inhalt                                                       | ab                |
| ------------ | ------------------------------------------------------------ | ----------------- |
| `welt`       | Formate, Meshing, Renderer; ohne Abhängigkeit vom Compositor | M1                |
| `viewer`     | die Welt als normales Linux-Fenster                          | M1                |
| `compositor` | Smithay; bindet `welt` ein                                   | M0 (Prototyp), M3 |
| `shell`      | Hotbar, Inventar, Kisten, Explorer                           | M4                |

Prototypen aus M0 liegen getrennt unter `prototypen/`.

## 3. Dateien

| Datei | Inhalt |
| ----- | ------ |
| [linux-umzug.md](linux-umzug.md) | Umstieg auf Linux, Voraussetzung für M0 |
| [m0-entscheidungen-und-prototypen.md](implementierung/m0-entscheidungen-und-prototypen.md) | Grundsatzentscheidungen, Fenster-Prototyp, Prototyp-Tests P1–P17 |
| [m1-weltviewer.md](implementierung/m1-weltviewer.md) | Welt lesen, Meshes, Navigation |
| [m2-minecraft-look.md](implementierung/m2-minecraft-look.md) | Licht, Himmel, Animationen, Transparenz, Pause |
| [m3-fenster-in-der-welt.md](implementierung/m3-fenster-in-der-welt.md) | Programme als bedienbare Screens an Wänden |
| [m4-shell-bedienung.md](implementierung/m4-shell-bedienung.md) | Hotbar, Inventar, Kisten, Explorer, Portal |
| [m5-weltsynchronisierung.md](implementierung/m5-weltsynchronisierung.md) | Weltupdate nach Spielende, Welt in Minecraft starten |
| [m6-performance-und-fernsicht.md](implementierung/m6-performance-und-fernsicht.md) | Mesh-Cache, Bildtaktung, Fernsicht |
| [m7-mod-und-shader-support.md](implementierung/m7-mod-und-shader-support.md) | Mod-Adapter, eingebaute Shader |
| [m8-echte-shell.md](implementierung/m8-echte-shell.md) | Login-Sitzung, Notfallmodus, Shell-Grundfunktionen |
| [m9-multiplayer.md](implementierung/m9-multiplayer.md) | gemeinsame Welt (nach dem Release) |
| [recherche-offen.md](implementierung/recherche-offen.md) | ausstehende Recherchen (R…) |
| [recherche-ergebnisse.md](implementierung/recherche-ergebnisse.md) | Funde abgeschlossener Recherchen mit Quellen |

## 4. Kritische Stellen im Überblick

| Stelle                                                                                       | Bewertung                                     | Meilenstein                     |
| -------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------- |
| Bedienbare Fenster auf 3D-Flächen mit Pop-ups, Dialogen, Fokus und Platzierung neuer Fenster | sehr schwer, Abbruchrisiko                    | M0 (Prototyp), M3               |
| Explicit Sync und dmabuf auf NVIDIA                                                          | schwer, Pflicht                               | M0                              |
| Blockdaten, die nicht in den Assets stehen                                                   | schwer, größter Zeitfresser                   | M1                              |
| Absturz des Compositors reißt alle Programme mit                                             | 2026 nicht allgemein lösbar; Architekturfrage | M0 (T3), M8                     |
| Physisches Verschieben von Dateien ohne Datenverlust                                         | mittel, Sicherheitsfrage                      | M4                              |
| Minecraft aus der Shell starten (Anmeldung)                                                  | mittel, hängt am Launcher                     | M5                              |
| Mob-Modelle                                                                                  | schwer, Modelle müssen selbst gebaut werden   | keinem zugeordnet (Abschnitt 5) |
| Look-Parität, Shader-Übersetzung, Fernsicht, Mods                                            | machbar, aber leicht endlos                   | M2, M6, M7                      |

**Schon entschieden** (in der Featureliste):
- Minecraft wird unter Linux gespielt.
- Offene Anwendungen erscheinen nur im eigenen Explorer.
- In der Pause halten Weltzeit und Animationen an, Fenster werden weiter gezeichnet.
- Die Freisteuerung baut eine feste Teilmenge der Minecraft-Bewegung nach.

## 5. Lücken im Gesamtplan

- **Mobs und KI-Stufe** (Featureliste, Abschnitt 3) stehen in keinem Meilenstein. Die Modelle der Java-Edition stehen im Code, nicht in den Assets; jedes Mob-Modell muss samt Laufanimation selbst gebaut werden. In welchem Meilenstein? Offen.
- **Shell-Grundfunktionen** wie Sperrbildschirm und Bildschirmfreigabe sind M8 zugeordnet; welche genau, ist offen (Featureliste, Frage 11).
