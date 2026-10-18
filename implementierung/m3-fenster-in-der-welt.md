# M3: Fenster in der Welt

**Planungsstand:** grob, nicht bereit.
- Voraussetzungen: die Ergebnisse von M0 (Fenster-Prototyp) und D6, D9, D10.
- Entscheidungen fehlen: T11 (m0), T12.

**Größe:** 6–12+ Wochen, Risiko sehr hoch. Das ist der größte Zeitfresser des Projekts.
**Voraussetzung:** M0 und M2. Der Compositor bindet die Crate `world` ein.
**Features:** [features.md](../features.md) §4 (Fenster in der Welt), Pause in §3. **Bedienung:** [bedienung.md](../bedienung.md) Screens, Drei Ansichten.
**M0 lesen:** [m0](m0-entscheidungen-und-prototypen.md) „Unbedingt beachten" (Puffer, Sync, Raycast, Pop-ups, Fenstergröße, Drosselung) ist die Grundlage dieser Datei; die M0-„Ergebnisse" gehen allem hier vor.

## Ziel

Die Features aus §4 produktiv umsetzen. Technisch heißt „mehrere Ansichten pro Fenster": **eine Fensterquelle, mehrere Darstellungen**, also eine Textur, die auf mehrere Quads und in ein 2D-Fenster gezeichnet wird.

## Kern: darauf kommt es an

1. **Fenster-Pipeline aus M0 produktiv machen:** dmabuf ohne Kopie, shm als Rückfall, Offscreen-Textur pro Fenster, Quad an der Wand.
2. **Eingabe pixelgenau, auch schräg** (P5), dazu Tastaturfokus, Scrollen, Zwischenablage und Drag & Drop.
3. **Pop-ups, Menüs, Tooltips, Dialoge und neue Fenster:** Ohne sie ist kein echtes Programm bedienbar. Wo neue Fenster und Dialoge erscheinen: D9.
4. **Wand-Logik:** Screens liegen auf Blockflächen und wachsen nur rechteckig bis zur Wandgrenze; Blöcke ohne Hitbox werden ignoriert.

## Zu entscheiden

| Nr. | Frage | Empfehlung bzw. Hinweise |
| --- | ----- | ------------------------ |
| T12 | Verankerung der Screens in der Layout-Datei | Anker = Blockposition + Seite + Rechteck in Blöcken. Fehlende Wand nach Weltupdate: F10 |
| D6, D9, D10 | Aufziehen und Mindestgröße, Ort neuer Fenster und Menüs, Fokus und Fluchttaste | bedienung.md. **D9 muss vor M3 entschieden sein** |
| F10 | Glas, Türen, Slabs, Treppen als Wandfläche? | features.md |

## Technische Grundlagen

Belegt in R7–R9 und R15.

### Puffer und Eingabe (Ergänzungen zu m0)

- **dmabuf:** `texture_from_dmabuf_fd` braucht das Feature `VULKAN_EXTERNAL_MEMORY_DMA_BUF`, danach `create_texture_from_hal`.
- **shm:** nur geänderte Bereiche hochladen.
- Den Zeiger zeichnet der Compositor selbst.
- Für die Freisteuerung gibt es `pointer_constraints` und `relative_pointer`.

### Pop-ups und Dialoge

- **Pop-ups:** `xdg_positioner`: Das Anker-Rechteck ist relativ zum Eltern-Surface; die Anpassung läuft in der Reihenfolge Flip, Slide, Resize.
- **Dialoge:**
  - `xdg_toplevel.set_parent` bildet die Hierarchie.
  - `xdg-dialog-v1` (`set_modal`) ist nur ein Hinweis; ob Smithay es hat, ist unsicher.

### Größe, Dekoration, Zwischenablage

- **Größe:** `configure` ist nur bei maximized und fullscreen bindend.
- **Zwischenablage und Drag & Drop:** Smithay `selection`; Drag & Drop nutzt denselben Raycast. Für den Explorer (M4) tritt der Compositor selbst als Datenquelle auf.
- **IME:** `text-input-v3` und `input-method-v2`; das Vorschlagsfenster sitzt am Cursor-Rechteck in der Wandebene.

### XWayland und Wine

- **xwayland-satellite** macht aus X11-Fenstern Wayland-Fenster. Programme mit festen Bildschirmkoordinaten können dabei brechen.
- **Ohne satellite:** Override-Redirect-Menüs haben Root-Koordinaten und müssen dem Eltern-Fenster zugeordnet werden.
- **Wine und Proton** laufen über XWayland, **Minecraft selbst** auch (m5).

### Drosselung, Vollbild, Spiele

- **Drosselung:** m0; Ziel etwa 1 Bild/s für entfernte und nicht sichtbare Fenster.
- **Vollbild:** Direct Scanout auf die Primary Plane, die Welt steht still.
- **Kopiergeschützte Videos** werden vermutlich nicht schwarz, weil Linux nur Widevine L3 hat (M0, P14).

## Unbedingt beachten

- **Hierarchie zuerst:** Ein Programm besteht aus Toplevel, Subsurfaces, Pop-ups und Dialogen. Diese Hierarchie von Anfang an sauber modellieren (Vorbild: das Panel-Modell von Stardust mit Child-Surfaces, Geometrie relativ zum Eltern-Surface, `z_order` und Eingaberegionen; nur die Technik).
- **Raycast-Reihenfolge:** Pop-ups, dann Dialoge, dann Fenster, dann Welt.
- **Fokus und Welt trennen:** Tasten gehen an das fokussierte Programm. Die Shell-Kürzel mit der Super-Taste fängt immer der Compositor ab. Eine Fluchttaste muss es immer geben (D10).
- **Wand-Erkennung:**
  - Sie braucht pro Block „hat Hitbox" und „ist Wandfläche", aus der Datenquelle von T7 in M1.
  - Ein Screen sitzt leicht vor der Blockfläche (wie ein Item-Frame, etwa 1/16 Block), damit es kein Z-Fighting gibt.
- **Lesbarkeit:**
  - Mip-Kette und anisotrope Filterung pro Fenster.
  - Mips nur bei Änderungen neu berechnen.
  - Gedrosselte Fenster kosten fast nichts.
- **Neue Fenster** dürfen nie unsichtbar oder unerreichbar werden. Jedes Toplevel ohne Wandplatz landet am vereinbarten Ort (D9), zum Beispiel in der Hotbar.
- **Schließen:** `xdg_toplevel.close` ist nur eine Bitte. Das Programm darf mit einem Dialog antworten („Speichern?"), der ebenfalls einen Platz braucht.
- **Absturzsicherheit:** Ein Fehler in Welt oder Renderer darf den Compositor nicht beenden, sonst sterben alle Programme mit (T3).

## Umfang (Bausteine)

1. Fenster-Pipeline aus M0 produktiv machen: dmabuf, shm, Sync, Offscreen-Textur, Mips.
2. Eingabe, Fokus, Zwischenablage, Drag & Drop, IME.
3. Pop-ups, Dialoge, Platzierung neuer Fenster (D9).
4. Wand-Erkennung, Platzieren mit Vorschau, Größe über die Eckpunkte ändern.
5. Minus und Kreuz (minimieren, schließen), Vollbild per Doppelklick, Rückkehr in die Welt.
6. XWayland über `xwayland-satellite`.
7. Drosselung nach Entfernung und Sichtbarkeit; Pause-Regel „neu zeichnen nur bei Fensteränderung".
8. Screens in der Layout-Datei pro Welt speichern (T12).

## Abnahme (ohne Codelesen prüfbar)

- **Alltag:** Der Nutzer arbeitet eine Stunde lang mit Firefox, einem Terminal und einem Dateimanager an den Wänden:
  - tippen, scrollen, Menüs und Dialoge benutzen
  - Text zwischen den Programmen kopieren
- **Größe:** Ein Screen lässt sich auf 3 × 2 Blöcke aufziehen und stoppt an der Wandkante. Eine Fackel an der Wand stört nicht.
- **Vollbild:** Die Welt-GPU-Last ist 0. Mit der vereinbarten Taste geht es zurück.
- **Drosselung:** Ein entferntes Video-Fenster läuft mit etwa 1 Bild pro Sekunde, gemessen.
- **Persistenz:** Nach einem Neustart hängen die Screens wieder an denselben Wänden.

## Referenzen

- Protokolle und Smithay: m0 „Referenzen"
- [Smithay `XwmHandler`](https://docs.rs/smithay/latest/smithay/xwayland/xwm/trait.XwmHandler.html), [niri und xwayland-satellite](https://github.com/niri-wm/niri/wiki/Xwayland)
- **Vorbilder, nur die Technik:**
  - Waylandcraft (GPL-3): `WorldPlane`, Tastaturfang
  - Stardust XR (GPL-2): Panel-Modell
  - SimulaVR (MIT): Leinwand pro Fenster mit Überstand für Pop-ups
