# M6: Performance und Fernsicht

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R28.
- Es fehlen das Ziel für die Renderdistanz und Messungen.

**Größe:** 4–10 Wochen, Risiko hoch bei der Fernsicht. Der Meilenstein kann ganz ans Ende geschoben werden.
**Voraussetzung:** M2 und M3.
**Features:** [features.md](../features.md) §7 (Performance, VRAM-Schätzung), Pause und Fernsicht in §3.

## Ziel

Die Ziele aus §7 erreichen und die Fernsicht aus §3 bauen.

## Kern: darauf kommt es an

1. **Bildtaktung** (T10) als zentrale Regel für alle Zustände:
   - Bewegung
   - stille Kamera mit Animationen
   - Pause
   - Vollbild
   - gedrosselte Fenster
2. **Mesh-Cache mit klarer Ungültigkeitsregel.** Der Schlüssel enthält Weltdaten (Chunk-Zeitstempel), Asset-Version, Resource Packs und die Version des Meshers.
3. **Fernsicht** entsteht im Weltupdate (M5) und wird gespeichert.

## Zu entscheiden

- Ziel für die Renderdistanz (normal und mit Fernsicht).
- Speicherformat und Ort der Fernsicht-Daten (R28).
- T10 im Detail, falls nach M2 noch offen.

## Technische Grundlagen

- **Drosselung der Fenster:** m0 „Unbedingt beachten".
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
