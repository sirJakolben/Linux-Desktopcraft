# M2: Minecraft-Look

**Planungsstand:** grob, nicht bereit.
- Recherche fehlt: R23–R25.
- Entscheidungen fehlen: T8, T10, erlaubte Abweichung vom Original.
- Fragen an den Nutzer fehlen: Featureliste, Fragen 5–7.

**Größe laut Featureliste:** 3–6 Wochen, Risiko mittel.
**Voraussetzung:** M1.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M2 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md), Quellen in [recherche-ergebnisse.md](recherche-ergebnisse.md).

## Ziel

Die Welt aus M1 sieht aus wie Minecraft:

- Smooth Lighting und Ambient Occlusion
- Sonnenzyklus, Himmel, Wolken und Nebel
- animierte Texturen
- Transparenz in der richtigen Reihenfolge
- Wasser- und Blätterbewegung (Umfang noch zu klären)
- Pausieren der Welt

## Kern: darauf kommt es an

1. **Licht:** Die gespeicherten Lichtwerte aus den Chunks werden pro Ecke gemittelt (Smooth Lighting) und mit Ambient Occlusion kombiniert. Das bestimmt am stärksten, ob es „nach Minecraft" aussieht.
2. **Transparenz korrekt:** Wasser, Glas und Eis sind durchscheinend und müssen sortiert werden. Blätter sind ausgeschnitten (Cutout) und brauchen eine eigene Mip-Behandlung.
3. **Geringe Last** (T10): Bei stiller Kamera nur 20 Hz für Animationen, im Pausezustand nichts. Hier wird das Versprechen „kaum Last" eingelöst.
4. **Erweiterbare Pipeline** (T8): Sie muss später eingebaute Shader aufnehmen können (M7), zum Beispiel G-Buffer und Nachbearbeitungs-Durchgänge.

## Entschieden

- **Behalten:** Sonnenzyklus, Himmel, Wolken, Shader, Wasser- und Blätterbewegung sowie **animierte Texturen**. Letztere sind dem Nutzer wichtig.
- **Pausieren:** Ist der Nutzer in einer Anwendung, halten Zeit, Animationen und Shader an. Im Vollbild entstehen keine Bilder. Ist die Welt sichtbar, wird nur neu gezeichnet, wenn sich ein Fenster ändert (ab M3). Danach läuft die Zeit exakt dort weiter, wo sie stand.
- **Shader:** Es gibt einen eingebauten, eigenen Shader. Übersetzte bekannte Shader folgen in M7.

## Zu entscheiden

| Nr. | Frage | Hinweise |
| --- | ----- | -------- |
| T8 | Pipeline forward oder deferred | deferred erleichtert spätere Shader-Pack-Nachbauten (G-Buffer), kostet aber mehr Speicher und Bandbreite |
| T10 | Bildtaktung | Empfehlung: Leerlauf 0 Bilder; stille Kamera mit Animationen 20 Hz; Bewegung mit voller Rate |
| – | erlaubte Abweichung vom Original | ohne Grenze ist „sieht genau aus wie Minecraft" Feintuning ohne Ende. Mit dem Nutzer ein Abnahmekriterium festlegen |
| F5 | Zählt schon der Fokus auf einem Screen an der Wand als „in einer Anwendung"? | Featureliste, Frage 5 |
| F6 | Tageszeit nach einem Weltupdate: aus dem Spiel oder aus der Shell? | Featureliste, Frage 6 |
| F7 | Wasser- und Blätterbewegung | Vanilla bewegt beides nicht, das machen nur Shader-Packs. Gehört es zum eingebauten Shader? (Featureliste, Frage 7) |

## Technische Grundlagen

Nach bisherigem Wissensstand, mit R23–R25 gegenprüfen.

- **Smooth Lighting:** Pro Eckpunkt einer Fläche werden 4 Lichtwerte gemittelt:
  - der Block vor der Fläche
  - die beiden seitlichen Nachbarn
  - die Ecke
- **Ambient Occlusion:** Je Ecke 3 Verdecker (Seite 1, Seite 2, Ecke); sind beide Seiten belegt, ist die Ecke voll verdeckt. Das Modell-Flag `ambientocclusion` beachten.
- **Diagonale:** Bei ungleichmäßigem AO die Viereck-Diagonale so wählen, dass keine Streifen entstehen.
- **Lightmap:** 16 × 16 Einträge, Himmelslicht (skaliert nach Tageszeit) gegen Blocklicht, dazu Gamma.
- **Tageszeit:** `DayTime` aus `level.dat`, 24 000 Ticks pro Tag, 20 Ticks pro Sekunde.
- **Himmel und Wetter:**
  - Texturen `textures/environment/sun.png`, `moon_phases.png` und `clouds.png`
  - Himmels- und Nebelfarbe aus den `effects` des Bioms (`sky_color`, `fog_color`)
  - Wetter aus `level.dat` (`raining`, `thundering`; prüfen)
- **Texturanimation:** `.mcmeta` mit `frametime` in Ticks und `interpolate`. Die Animationsframes werden im Texturatlas im 20-Hz-Takt aktualisiert, und zwar nur, wenn eine animierte Textur sichtbar ist.
- **Wasser:** Texturen `water_still` und `water_flow`, Färbung über die `water_color` des Bioms, Fließrichtung aus dem Blockstate `level` der Nachbarn.
- **Render-Layer:** solid, cutout, cutout_mipped und translucent. Welcher Block in welchen Layer gehört, steht im Code, nicht in den Assets. Die Daten kommen deshalb aus der Quelle von T7.

## Unbedingt beachten

- **Transparenz:** Durchscheinende Flächen pro Chunk-Section von hinten nach vorn sortieren (wie Sodium) oder Order-Independent Transparency verwenden. Die Sortierung nur neu machen, wenn sich die Kamera merklich bewegt.
- **Cutout und Mipmaps:** Bei ausgeschnittenen Texturen lassen Mipmaps Blätter in der Ferne verschwinden. Dagegen helfen Alpha-to-Coverage oder angepasste Alpha-Werte in den Mip-Stufen.
- **Texturatlas:** Ohne Rand um jede Kachel bluten die Farben in den Mip-Stufen (Padding).
- **Pausieren:**
  - Weltzeit und Animationen exakt anhalten, ohne Zeitsprung beim Fortsetzen.
  - Ein unverändertes Bild wird nicht neu gezeichnet.
  - Die Pause-Logik zentral halten, damit M3 die Regel „Fenster ändern sich → neu zeichnen" einhängen kann.
- **Look-Parität ohne Ende:** Nur bis zum vereinbarten Abnahmekriterium arbeiten, dann aufhören.

## Umfang (Bausteine)

1. Lichtwerte im Mesher: Smooth Lighting und AO pro Eckpunkt.
2. Lightmap und Tageszeit.
3. Himmel: Sonne, Mond, Sterne, Farbverlauf.
4. Wolken.
5. Nebel.
6. Animierte Texturen.
7. Transparenz-Sortierung.
8. Wasserdarstellung.
9. Wasser- und Blätterbewegung (nach Klärung von F7).
10. Pausieren mit Bildtaktung (T10).

## Abnahme (ohne Codelesen prüfbar)

- **Vergleich mit dem Spiel:** Screenshots an gleicher Position und Tageszeit (Mittag, Sonnenuntergang, Nacht mit Fackeln) neben dem Spiel. Der Nutzer beurteilt sie nach dem vereinbarten Kriterium.
- **Animationen:** Wasser, Lava, Feuer und Portale bewegen sich.
- **Transparenz:** Glas vor Wasser stimmt, Blätter verschwinden in der Ferne nicht.
- **Last:**
  - Stille Kamera: höchstens 20 Bilder pro Sekunde, GPU-Last gemessen.
  - Pause: 0 Bilder pro Sekunde.
  - Beim Fortsetzen springt die Tageszeit nicht.

## Offene Fragen an den Nutzer

- Featureliste, Fragen 5, 6, 7.
- Die erlaubte Abweichung vom Original (Planungsstand von M2 in der Featureliste).

## Referenzen

- Minecraft-Wiki: Seiten zu Light, Daylight cycle und Resource pack (`.mcmeta`)
- 0fps: „Ambient occlusion for Minecraft-like worlds" (Algorithmus für AO)
- Sodium: Technik der Transparenz-Sortierung (LGPL, nur die Technik)
