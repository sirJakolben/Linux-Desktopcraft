# Implementierungsplan

Übergreifende technische Entscheidungen, Architektur, Meilensteine. Für jeden Meilenstein mitlesen.

## 1 Technische Entscheidungen

### Entschieden

- **T5 Entwicklungsweg:** direkt unter Linux; unter Windows lauffähig zu sein ist kein Ziel. Windows bleibt nur übergangsweise im Dual Boot (für die Mod „Smart Villagers").
- **Qualität** (folgt daraus, dass Claude Code programmiert):
  - `cargo fmt` und `cargo clippy -- -D warnings` sauber; Tests gehören zu jedem Baustein.
  - `unsafe` nur in wenigen kleinen Modulen (dmabuf-Import, Vulkan-Interop), jedes mit Kommentar zur Sicherheitsbegründung.
- **T17 Lizenz 0BSD:** `LICENSE` kommt mit dem ersten Code ins Repository. Abhängigkeiten unter freizügigen Lizenzen (MIT, Apache, BSD, Zlib, ISC) sind in Ordnung, Copyleft nicht (Nutzer 18.10.2026).
- **T18 Installationsort:** Linux in 500 GB auf der NVMe-SSD mit eigener EFI-Partition (die von Windows hat nur 100 MB), damit Windows später ohne Umbau gelöscht werden kann. Details: [linux-umzug.md](linux-umzug.md).
- **Hardware:** RTX 5060 Ti (Blackwell) mit 16 GB VRAM; NVMe-SSD WD_BLACK SN850X 2 TB; HDD Toshiba 1 TB (D:).

### Verzeichnis aller T-Entscheidungen

Details, Empfehlung und Hinweise stehen in der genannten Meilenstein-Datei.

| Nr. | Thema | Status | Datei |
| --- | ----- | ------ | ----- |
| T1 | Sprache und Grafik-API | entschieden: Rust + wgpu | [m0](implementierung/m0-entscheidungen-und-prototypen.md) |
| T2 | Compositor-Bibliothek | entschieden: Smithay | m0 |
| T3 | Prozessaufteilung | entschieden: ein Prozess | m0 |
| T4 | Linux-Distribution und Desktop | entschieden: Fedora mit GNOME | m0 |
| T5 | Entwicklungsweg | entschieden | hier |
| T6 | unterstützte Minecraft-Versionen, Asset-Version | offen | [m1](implementierung/m1-weltviewer.md) |
| T7 | Quelle für Blockdaten außerhalb der Assets | offen | m1 |
| T8 | Render-Pipeline forward/deferred | offen | [m2](implementierung/m2-minecraft-look.md) |
| T9 | Datenstruktur Chunks/Meshes, Mesh-Cache | offen | m1 |
| T10 | Bildtaktung je Zustand | offen, Empfehlung vorhanden; erster Performance-Durchgang | m0 |
| T11 | Fensterauflösung und Größenaushandlung | offen, Empfehlung vorhanden | m0 |
| T12 | Identität der Welt (m1), Verankerung der Screens (m3) | offen | m1, [m3](implementierung/m3-fenster-in-der-welt.md) |
| T13 | Bewegungsmodell (F4) | offen | m1 |
| T14 | Welten finden und lesen (m1), Weltcache und Änderungserkennung (m5) | offen | m1, [m5](implementierung/m5-weltsynchronisierung.md) |
| T15 | Format der Einstellungen | offen | m1 |
| T16 | Minecraft starten: Launcher steuern oder eigener | offen | m5 |
| T17 | Lizenz 0BSD | entschieden | hier |
| T18 | Installationsort | entschieden | hier |
| T19 | Sprache im Code | entschieden: Code Englisch, Sichtbares Deutsch | m0 |

## 2 Architektur (gilt mit T1–T3)

Cargo-Workspace:

| Crate | Inhalt | ab |
| ----- | ------ | -- |
| `world` | Formate, Meshing, Renderer; ohne Abhängigkeit vom Compositor | M1 |
| `viewer` | die Welt als normales Linux-Fenster (M1, M2) | M1 |
| `compositor` | Smithay; bindet `world` ein | M0 (Prototyp), M3 |
| `shell` | Hotbar, Inventar, Kisten, Explorer | M4 |

Während der Entwicklung läuft alles als Fenster in einem Host-Desktop, der die Super-Taste abfängt. Die Modifier-Taste der Shell ist deshalb konfigurierbar.

## 3 Meilensteine

Reihenfolge (Nutzer 18.10.2026): M0 bis A5 → **M8 Teil 1**: eigene Login-Sitzung mit der Testszene statt der Welt (siehe m8) → M1 und M2 als normale Linux-Anwendung → M3 → M4, M5 → M6, M7, Rest von M8 → M9 nach dem Release.

**Performance-Durchgang** (Nutzer 18.10.2026): Performance ist sonst kein Abnahmekriterium. Nach jedem Meilenstein (auch nach M8 Teil 1): Last im Leerlauf, Bilder/s und VRAM messen, Ausreißer beheben, Werte in der Meilenstein-Datei festhalten. Der erste Durchgang übernimmt A6 aus m0 (P11, P12, T10).

| Datei | Inhalt |
| ----- | ------ |
| [linux-umzug.md](linux-umzug.md) | Umstieg auf Linux, Voraussetzung für M0 |
| [m0](implementierung/m0-entscheidungen-und-prototypen.md) | Grundsatzentscheidungen, Fenster-Prototyp, Prototyp-Tests P1–P17, NVIDIA- und Fenstertechnik |
| [m1](implementierung/m1-weltviewer.md) | Welt lesen, Meshes, Navigation |
| [m2](implementierung/m2-minecraft-look.md) | Licht, Himmel, Animationen, Transparenz, Pause |
| [m3](implementierung/m3-fenster-in-der-welt.md) | Programme als bedienbare Screens an Wänden |
| [m4](implementierung/m4-shell-bedienung.md) | Hotbar, Inventar, Kisten, Explorer, Portal |
| [m5](implementierung/m5-weltsynchronisierung.md) | Weltupdate nach Spielende, Welt in Minecraft starten |
| [m6](implementierung/m6-performance-und-fernsicht.md) | Mesh-Cache, Bildtaktung, Fernsicht |
| [m7](implementierung/m7-mod-und-shader-support.md) | Mod-Adapter, eingebaute Shader |
| [m8](implementierung/m8-echte-shell.md) | Login-Sitzung, Notfallmodus, Shell-Grundfunktionen |
| [m9](implementierung/m9-multiplayer.md) | gemeinsame Welt (nach dem Release) |

## 4 Kritische Stellen

| Stelle | Bewertung | Meilenstein |
| ------ | --------- | ----------- |
| Bedienbare Fenster auf 3D-Flächen mit Pop-ups, Dialogen, Fokus und Platzierung neuer Fenster | sehr schwer, Abbruchrisiko | M0 (Prototyp), M3 |
| Explicit Sync und dmabuf auf NVIDIA | schwer, Pflicht | M0 |
| Blockdaten, die nicht in den Assets stehen | schwer, größter Zeitfresser | M1 |
| Absturz des Compositors reißt alle Programme mit | 2026 nicht allgemein lösbar; Architekturfrage | M0 (T3), M8 |
| Physisches Verschieben von Dateien ohne Datenverlust | mittel, Sicherheitsfrage | M4 |
| Eigener Datei-Explorer | eigenes Teilprojekt, noch nicht geschätzt | M4 |
| Minecraft aus der Shell starten (Anmeldung) | mittel, hängt am Launcher | M5 |
| Mob-Modelle | schwer, Modelle müssen selbst gebaut werden | keinem zugeordnet (§5) |
| Look-Parität, Shader-Übersetzung, Fernsicht, Mods | machbar, aber leicht endlos | M2, M6, M7 |

## 5 Lücken im Gesamtplan

- **Mobs und KI-Stufe** (features.md §3) stehen in keinem Meilenstein. Die Modelle der Java Edition stehen im Code, nicht in den Assets; jedes Mob-Modell muss samt Laufanimation selbst gebaut werden. Welcher Meilenstein: offen.
- **Shell-Grundfunktionen** (F11) sind M8 zugeordnet; welche genau, ist offen.
