# M1: Weltviewer

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R12, R17–R22 (siehe [recherche-offen.md](recherche-offen.md)).
- Entscheidungen fehlen: T6, T7, T9, T12, T13, T14, T15.

**Größe:** 3–6 Wochen, Risiko mittel.
**Voraussetzung:** M0 abgeschlossen (T1–T3 entschieden).
**Features:** [features.md](../features.md) §2 Navigation, §3 Welt (Umfang, Look), §6 eigene Daten. **Bedienung:** [bedienung.md](../bedienung.md) Tasten, Kamera und Waypoints.

## Ziel

Die echte Minecraft-Java-Welt des Nutzers **als normales Linux-Fenster** anzeigen und begehen (Crate `viewer`, noch ohne Compositor):

- Regionsdateien lesen
- Chunks zu Meshes umrechnen
- Blockmodelle und Texturen aus der lokalen Installation laden
- Waypoint-Bewegung, Kamera und Freisteuerung

Licht, Himmel und Animationen kommen erst in M2. In M1 genügt eine einfache Schattierung pro Fläche. Mobs gehören nicht zu M1.

## Kern: darauf kommt es an

1. **Datenpfad:** Regionsdatei → Chunk-NBT → Paletten-Index → Blockstate → Modell → Mesh. Er muss robust sein gegen Versionsunterschiede und unbekannte Blöcke: dann ein sichtbarer Platzhalter und ein Logeintrag, **nie ein Absturz**.
2. **Blockdaten, die nicht in den Assets stehen** (T7). Das ist der größte Zeitfresser:
   - Verdeckung: welcher Block voll und undurchsichtig ist (fürs Weglassen verdeckter Flächen)
   - Render-Layer
   - Kollisionsformen (Freisteuerung jetzt, Screen-Skalierung in M3)
   - Flüssigkeiten
   - Block-Entities (Kisten, Schilder, Betten, Banner, Köpfe)
   - Biomfarben
   Dazu kosten Sonderformen Zeit (Treppen, Zäune, Türen, Mehrteilblöcke über Blockstates).
3. **Streaming:** Chunks werden asynchron in Worker-Threads gebaut, von innen nach außen. Das GPU-Upload passiert im Render-Thread ohne Ruckler. Die Welt endet an nicht generierten Chunks.
4. **`welt` bleibt unabhängig vom Compositor**, damit M3 die Crate unverändert einbinden kann.

Fallback-Texturen sind eigene (keine Mojang-Dateien).

## Zu entscheiden

| Nr. | Frage | Hinweise |
| --- | ----- | -------- |
| T6 | Unterstützte Versionen (DataVersion-Bereich); welche Asset-Version bei mehreren Installationen | Der Nutzer spielt aktuell 26.x; genaue Version: F8. Ältere Chunks in alten Formaten: lesen, umrechnen oder als Platzhalter zeigen? |
| T7 | Quelle für die fehlenden Blockdaten | Optionen siehe unten. Lizenz und Aktualität prüfen (R20) |
| T9 | Datenstruktur für Chunks und Meshes, Format des Mesh-Caches | der Cache muss in M6 ungültig gemacht werden können (Weltdaten, Asset-Version, Resource Packs, Mesher-Version) |
| T12 | Identität einer Welt für die Layout-Datei | Weltordner haben keine feste ID; Umbenennen bricht die Zuordnung. Verankerung der Screens: m3 |
| T13 | Teilmenge der Bewegung | F4 |
| T14 | Wo liegen die Welten (R12), und wird aus dem Original oder aus einer Kopie gelesen? | Cache und Änderungserkennung: m5 |
| T15 | Format der Einstellungen | – |

**Optionen für T7:**
- Mojangs Datengenerator lokal ausführen (`--reports`, steckt im server.jar; von den Richtlinien nicht erwähnt)
- `azalea-registry` und `azalea-block` (aktiv, MC 26.2)
- PrismarineJS minecraft-data
- eine eigene Tabelle

## Technische Grundlagen

Nach bisherigem Wissensstand, mit R17–R20 für 26.x gegenprüfen.

### Regionsdatei `r.<x>.<z>.mca`

- **Inhalt:** 32×32 Chunks.
- **Kopf:** 4 KiB mit 1024 Einträgen je 4 Byte (3 Byte Offset in 4-KiB-Sektoren, 1 Byte Sektorzahl), dann 4 KiB mit 1024 Zeitstempeln (Sekunden). Die Zeitstempel nutzt M5 für den Chunk-Abgleich.
- **Chunk-Daten:** 4 Byte Länge, 1 Byte Kompression (1 gzip, 2 zlib, 3 roh, 4 LZ4 seit 1.20.5), dann NBT.
- **Ausgelagerte Chunks:** Ist Bit 128 der Kompression gesetzt, steht der Chunk in `c.<x>.<z>.mcc`.
- **Ordner:**
  - Oberwelt `region/`, Nether `DIM-1/region/`, End `DIM1/region/` (Struktur für 26.x prüfen).
  - Entities liegen seit 1.17 getrennt in `entities/`.

### Chunk-NBT (seit 1.18, ohne `Level`-Hülle)

- **Felder:** `DataVersion`, `xPos`, `zPos`, `yPos`, `Status`, `sections`, `block_entities`, `Heightmaps`, `isLightOn`.
- **Status:** Nur `minecraft:full` (früher `full`) gilt als vorhanden. Andere Status sind halb generierte Ränder und zählen als nicht vorhanden.
- **Section:**
  - `Y` (Byte)
  - `block_states` = {`palette`: [{`Name`, `Properties`}], `data`: long[]}
  - `biomes` = {`palette`, `data`}
  - optional `BlockLight` und `SkyLight` (je 2048 Byte, 4 Bit pro Block)
- **Gepackte Indizes:**
  - Blöcke: max(4, ⌈log₂(Palettengröße)⌉) Bit.
  - Biome: ⌈log₂(Palettengröße)⌉ Bit, Auflösung 4×4×4.
  - Seit 1.16 reicht kein Eintrag über eine Long-Grenze. Bei einer Palette mit nur einem Eintrag fehlt `data`.
  - Reihenfolge des Index: `y*256 + z*16 + x`.
- **Höhen:** Oberwelt Y −64 bis 319 (24 Sections), der Nether hat 0–255.
- **Licht:** Minecraft speichert Licht in den Chunks. Die Werte auslesen statt selbst berechnen (ob sie vorhanden sind, zeigt `isLightOn`).
- **Alte Chunks:** Minecraft wandelt Chunks erst beim Laden um. Gebiete, die seit einem Versionswechsel nie wieder besucht wurden, liegen **im alten Format** in der Welt. Deshalb wird die DataVersion pro Chunk geprüft (T6).

### `level.dat`

- gzip-NBT, Wurzel `Data`
- `DataVersion`, `Version.Name`, `LevelName`
- `DayTime` (24 000 Ticks pro Tag, 20 Ticks pro Sekunde)
- `Player.Pos`, `Player.Rotation`, `Player.Dimension` bei Singleplayer
- Spawnpunkt (Feldnamen für 26.x prüfen)

### Assets aus `.minecraft/versions/<v>/<v>.jar` (ZIP)

- **Blockstates** (`assets/minecraft/blockstates/*.json`):
  - entweder `variants` (Schlüssel `prop=wert,…`; `model`, `x`, `y`, `uvlock`, `weight`; Arrays sind gewichtete Varianten)
  - oder `multipart` (`when` mit `OR`, dazu `apply`)
- **Modelle** (`models/block/*.json`):
  - `parent`-Kette, `textures` mit `#`-Referenzen, `ambientocclusion`
  - `elements`: `from`/`to` (0–16), `rotation` (`origin`, `axis`, `angle` in Schritten von ±22,5°, `rescale`), `shade`
  - `faces`: `uv`, `texture`, `cullface`, `rotation`, `tintindex`
- **Texturen:** `textures/block/*.png`. Animationen über `*.png.mcmeta`: `frametime` in Ticks, `frames`, `interpolate`.
- **Biomfarben:**
  - Colormaps `textures/colormap/grass.png` und `foliage.png` (dazu weitere ab 1.21.x prüfen)
  - Biomdaten `data/minecraft/worldgen/biome/*.json` mit `effects` (`grass_color`, `foliage_color`, `water_color`, `grass_color_modifier`)
- **Nicht in den Assets:** Verdeckung, Render-Layer (solid / cutout / translucent), Kollisionsformen, Flüssigkeitsdarstellung und Block-Entities. Diese Daten kommen aus T7.

### Weitere Grundlagen

- **Crates:**
  - fastnbt und simdnbt (aktiv)
  - fastanvil (vor 1.0; ob es 26.x liest, prüft P16)
  - azalea-Teilcrates für Blockdaten
- **Spieler:** Augenhöhe 1,62, Hitbox 0,6 × 1,8 Blöcke. Bewegungskonstanten aus dem Minecraft-Wiki nachbauen.

## Unbedingt beachten

- **Nie abstürzen wegen Weltdaten:** unbekannte Blöcke, fehlende Modelle, kaputte Chunks und alte Formate ergeben Platzhalter und einen Zähler im Log.
- **Falsche Verdeckung** erzeugt Löcher oder unsichtbare Wände. Die Verdeckung hängt an den Daten aus T7, nicht an Vermutungen.
- **Die Originalwelt nie sperren.** Läuft Minecraft gerade (`session.lock`), wird nur gelesen. Der saubere Weg über einen Cache kommt in M5.
- **Bildtaktung:** Auch der Viewer zeichnet nur bei Bedarf (T10). Ohne Kamerabewegung gibt es keine neuen Bilder, Animationen kommen erst in M2.
- **Tests:** Mojang-Dateien dürfen nicht ins Repository. Unit-Tests bauen ihre NBT-Daten selbst; Integrationstests nutzen die lokale Installation, wenn vorhanden, sonst werden sie übersprungen.

## Umfang (Bausteine)

1. **Assets:** Installation finden, Version wählen (T6), JAR lesen, Blockstates und Modelle auflösen, Texturatlas mit Mip-Rand bauen.
2. **Welt:** Regionsdateien und Chunks lesen, Paletten und `level.dat` dekodieren.
3. **Mesher:** verdeckte Flächen weglassen, Tint, Flüssigkeiten, Platzhalter für Block-Entities.
4. **Renderer:** einfache Schattierung pro Fläche, Frustum-Culling, Render-Layer.
5. **Streaming:** Worker-Pool, Ladereihenfolge von innen nach außen, Nebel als Kaschierung, Entladen außerhalb der Distanz.
6. **Navigation:** Waypoint-Graph, Laufen zwischen Waypoints, Kameramodi, Freisteuerung mit Kollision (T13).
7. **Layout-Datei pro Welt** (T12): Waypoints speichern und laden.

## Abnahme (ohne Codelesen prüfbar)

- **Erkennbarkeit:** Der Nutzer öffnet seine Welt und erkennt sein Haus. Ein Screenshot an der Position aus `level.dat` neben einem Screenshot aus dem Spiel zeigt dieselben Blöcke und Texturen.
- **Robustheit:** Ein Rundgang über die ganze Welt endet ohne Absturz. Die Zahl unbekannter Blöcke steht im Log.
- **Ladezeit:** Startzeit bis zum ersten Bild und bis zur vollen Renderdistanz ist gemessen und notiert.
- **Waypoints:** Setzen, verbinden und ablaufen funktioniert; nach einem Neustart sind sie noch da.
- **Freisteuerung:** Man läuft nicht durch Wände.
- **Leerlauf:** Ohne Bewegung entstehen 0 neue Bilder.

## Offene Fragen an den Nutzer

F4, F8; D2, D3, D4, D5, D11.

## Referenzen

- **Minecraft-Wiki:** Seiten zu Region file format, Chunk format, Model und Blockstates
- **Crates:** [fastnbt](https://github.com/owengage/fastnbt), [simdnbt und azalea](https://github.com/azalea-rs/azalea)
- **Renderer außerhalb des Spiels:** BlueMap, Chunky, deepslate (misode)
- **Nur Technik, kein Code** (Copyleft):
  - wgpu-mc (MPL-2.0)
  - [Pomme](https://github.com/PommeMC/Client) (GPL-3)
