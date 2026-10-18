# Offene Recherchen

Sortiert nach Meilenstein; vor dem Konkretisieren eines Meilensteins abarbeiten. Ablauf nach Abschluss: CLAUDE.md, „Pflege".

## M0: Entscheidungen und Prototypen

- **R32** Waylandcraft studieren (nur die Technik, GPL-3.0, keinen Code übernehmen):
  - dmabuf-Import über EGL
  - `WorldPlane`: Raycast und Pixel pro Block
  - Tastaturfang mit `G` und `Alt+Q`
  - Umgang mit NVIDIA-Problemen
- **R33** Unsichere Punkte aus der M0-Recherche, im Prototyp klären:
  - Braucht wgpu den Import fremder Semaphoren, oder reicht Smithays `DrmSyncPointBlocker`? Wann wird der Release-Punkt signalisiert?
  - Bildet `xwayland-satellite` X11-Override-Redirect-Fenster als `xdg_popup` ab?
  - Ist `xdg-dialog-v1` in Smithay vorhanden?
  - Wie kommen wgpu-Bilder auf die DRM-Ausgabe von Smithay (Prototyp-Test P15)?

## M1: Weltviewer

- **R12** Weltdateien finden:
  - Minecraft-Ordner unter Linux: offizieller Launcher, Prism Launcher, Flatpak.
  - Für die vorläufige Dual-Boot-Alternative die Windows-Partition (NTFS) lesen (Hinweise in m5).
- **R17** Welt- und Chunkformat im Stand 26.x:
  - Anvil-Regionsdateien, Chunk-NBT, Kompression (zlib, LZ4, ausgelagerte `.mcc`-Dateien)
  - DataVersion und Unterschiede zwischen den Versionen
  - Ordner der Dimensionen
  - gespeichertes Licht
  - Chunk-Status (nur fertig generierte Chunks gelten)
  - Chunks in altem Format, die nie neu geladen wurden
  - Die Grundlagen nach bisherigem Wissensstand stehen in [m1-weltviewer.md](m1-weltviewer.md); sie sind gegenzuprüfen.
- **R18** `level.dat` im Stand 26.x: Spielerposition, Version, Spawnpunkt, Tageszeit.
- **R19** Assets aus der lokalen Installation:
  - Blockstates und Modelle: Elemente, Rotation, UV-Lock, Multipart, gewichtete Varianten, Tint-Indizes
  - Texturen und Animationsdaten (`.mcmeta`)
  - Resource Packs
  - Ordner `items/` seit 1.21.4
- **R20** Was nicht in den Assets steht:
  - Verdeckung (voll und undurchsichtig) und Render-Layer
  - Kollisionsformen
  - Flüssigkeiten und Block-Entities
  - Biomfarben
  - Mögliche Quellen: Mojangs Datengenerator (`--reports`, steckt im server.jar, lokal ausführen), `azalea-registry` und `azalea-block`, PrismarineJS minecraft-data, Burger, misode/mcmeta. Prüfen: Lizenz und Aktualität jeder Quelle.
- **R21** Bestehende Minecraft-Renderer außerhalb des Spiels:
  - Wie lösen BlueMap, Chunky, Mineways, deepslate und prismarine-viewer Blockmodelle, Sonderblöcke und Biomfarben?
  - Liest fastanvil das Format von 26.x?
- **R22** Meshing:
  - verdeckte Flächen weglassen
  - Greedy Meshing und seine Grenzen (AO, Texturen)
  - Ansätze von Sodium
  - Format des Mesh-Caches

## M2: Minecraft-Look

- **R23** Licht: Smooth Lighting und Ambient Occlusion wie in Vanilla, Lightmap (Himmels- und Blocklicht nach Tageszeit), Nebel.
- **R24** Himmel und Wetter: Sonne, Mond, Sterne, Himmelsfarbe, Wolken (Stand 26.x).
- **R25** Wasser, Blätter, Transparenz:
  - Wie bewegen Shader-Packs Wasser und Blätter?
  - Transparente Flächen sortieren (Sodium) oder Order-Independent Transparency.
  - Mipmaps bei ausgeschnittenen Texturen (Blätter).

## Spätere Meilensteine (nur grob, damit die Architektur nichts verbaut)

- **R27 (M4)** Dateisystem:
  - Ordner überwachen (inotify/fanotify)
  - Item-Positionen speichern (erweiterte Dateiattribute, versteckte Datei oder Datenbank)
  - Verhalten bei Änderungen außerhalb des eigenen Explorers
  - Papierkorb nach freedesktop-Spezifikation
- **R28 (M6)** Fernsicht: Wie erzeugt und speichert Distant Horizons seine LODs?
- **R29 (M7)** Mods und Shader:
  - Formate der Modloader (Fabric, NeoForge)
  - Aufbau von Iris- und OptiFine-Shader-Packs und ihre Lizenzen
  - Folgen von Mojangs Umstieg auf Vulkan
- **R30 (M8)** Shell-Grundfunktionen:
  - Benachrichtigungen (`org.freedesktop.Notifications`), Audio (PipeWire), mehrere Monitore, Energiesparen
  - Sperrbildschirm, Bildschirmfreigabe (xdg-desktop-portal)
- **R31 (M5)** Minecraft aus der Shell starten:
  - Quick Play (`--quickPlaySingleplayer`)
  - Kommandozeile von Prism Launcher, offizieller Launcher, eigener Launcher (Microsoft-Anmeldung)
  - Läuft Minecraft unter Wayland nativ oder über XWayland?
