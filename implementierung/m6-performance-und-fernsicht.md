# M6: Performance und Fernsicht

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R28.
- Es fehlen das Ziel für die Renderdistanz und Messungen. Der VRAM ist bekannt: 16 GB.

**Größe laut Featureliste:** 4–10 Wochen, Risiko hoch bei der Fernsicht. Der Meilenstein kann ganz ans Ende geschoben werden.
**Voraussetzung:** M2 und M3.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M6 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md).

## Ziel

- **Welten laden in wenigen Sekunden** von der SSD, dank zwischengespeicherter Meshes.
- **Chunks** werden von innen nach außen nachgeladen, Nebel kaschiert den Rand.
- **Gezeichnet wird nur bei Bedarf.** Fenster außerhalb des Blickfelds werden gedrosselt.
- **Fernsicht** mit Detailstufen im Stil von Distant Horizons.

## Kern: darauf kommt es an

1. **Bildtaktung** (T10) als zentrale Regel für alle Zustände:
   - Bewegung
   - stille Kamera mit Animationen
   - Pause
   - Vollbild
   - gedrosselte Fenster
2. **Mesh-Cache mit klarer Ungültigkeitsregel.** Der Schlüssel enthält Weltdaten (Chunk-Zeitstempel), Asset-Version, Resource Packs und die Version des Meshers.
3. **Fernsicht:** Sie zeigt nur bereits geladenes oder besuchtes Gelände. Berechnet wird sie **einmal beim Weltupdate** (M5) und dann gespeichert.

## Entschieden

- Gezeichnet wird nur, wenn nötig. Im Vollbild oder in der Pause wird die Welt nicht gerendert.
- Fertige Meshes werden gespeichert. Ziel: wenige Sekunden Ladezeit.
- Es ist immer nur die aktive Welt geladen; andere gibt es nur als Vorschaubild.
- Fenster außerhalb des Blickfelds laufen mit etwa 1 Bild pro Sekunde.
- Mobs sind standardmäßig aus, die KI ist vereinfacht.
- Hardware: RTX 5060 Ti mit 16 GB VRAM.

## Zu entscheiden

- Ziel für die Renderdistanz (normal und mit Fernsicht).
- Speicherformat und Ort der Fernsicht-Daten (R28).
- T10 im Detail, falls nach M2 noch offen.

## Technische Grundlagen

- **Schätzung aus der Featureliste (ungemessen):** 100–300 MB VRAM bei etwa 12 Chunks Renderdistanz, die Fernsicht zusätzlich 1–3 GB.
- **Drosselung der Fenster** über zurückgehaltene Frame-Callbacks und früher gelöste `fifo-v1`-Barrieren (siehe M3).
- **Leerlauf:** 0 Redraws sind möglich (M0, P12).

## Unbedingt beachten

- **Erst messen, dann optimieren.** Messwerte für Ladezeit, VRAM, GPU-Last im Leerlauf und bei stiller Kamera festhalten.
- **Ein veralteter Cache darf nie eine falsche Welt zeigen.** Im Zweifel neu bauen.
- **Fernsicht ist leicht endlos.** Den Umfang vorher mit dem Nutzer festlegen.

## Abnahme

Konkretisieren, wenn der Meilenstein geplant wird: Ladezeit, VRAM und Last im Leerlauf als Zahlen, Fernsicht sichtbar bis zur Zielweite.

## Referenzen

- Distant Horizons (Technik der LOD-Erzeugung, R28)
- Sodium (Meshing und Sortierung, nur die Technik)
