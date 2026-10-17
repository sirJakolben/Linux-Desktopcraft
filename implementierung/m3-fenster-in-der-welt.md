# M3: Fenster in der Welt

**Planungsstand:** grob, nicht bereit.
- Voraussetzungen: die Ergebnisse von M0 (Fenster-Prototyp) und die Bedienfragen im Designplan (Fragen 6, 9, 10).
- Entscheidungen fehlen: T11, T12.

**Größe laut Featureliste:** 6–12+ Wochen, Risiko sehr hoch. Das ist der größte Zeitfresser des Projekts.
**Voraussetzung:** M0 und M2. Der Compositor bindet die Crate `welt` ein.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M3 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md), Quellen in [recherche-ergebnisse.md](recherche-ergebnisse.md). Die Ergebnisse aus [M0](m0-entscheidungen-und-prototypen.md), Abschnitt „Ergebnisse", gelten vorrangig vor allem, was hier steht.

## Ziel

Echte Programme erscheinen als **Screens an den Wänden** der Welt: als Live-Textur, bedienbar mit Maus und Tastatur, auch aus der Ferne sichtbar. Dazu gehören:

- Vollbild
- Größe über mehrere Blöcke ändern
- Minimieren in die Hotbar (als Icon, siehe M4)
- Schließen (Wand-Screens werden zu Verknüpfungen mit letztem Screenshot)
- **Mehrere Wand-Screens pro Fenster:** dasselbe Programmfenster kann mehrfach an Wänden hängen und gleichzeitig als Desktop-Fenster und Icon existieren; der zuletzt benutzte Wand-Screen ist das Prime-Fenster (Teleport-Ziel). Das heißt: **eine Fensterquelle, mehrere Darstellungen**, also eine Textur, die auf mehrere Quads und in ein 2D-Fenster gezeichnet wird.
- Platzieren mit Vorschau

## Kern: darauf kommt es an

1. **Fenster-Pipeline:** Puffer der Programme (dmabuf ohne Kopie, shm als Rückfall), dann pro Fenster eine Offscreen-Textur mit dem ganzen Surface-Baum und Mip-Kette, dann ein Quad an der Wand. Explicit Sync auf NVIDIA ist Pflicht.
2. **Eingabe:** Raycast → UV → Fenstergeometrie → `surface_at` → `wl_pointer`. Er muss pixelgenau sein, auch schräg. Dazu Tastaturfokus, Scrollen, Zwischenablage und Drag & Drop.
3. **Pop-ups, Menüs, Tooltips, Dialoge und neue Fenster:** Ohne sie ist kein echtes Programm bedienbar. Wo neue Fenster und Dialoge erscheinen, legt der Nutzer im Designplan fest (Frage 9).
4. **Wand-Logik:** Screens liegen auf Blockflächen und wachsen nur rechteckig bis zur Wandgrenze. Blöcke ohne Hitbox werden ignoriert.

## Entschieden

- **Screens:**
  - Programme erscheinen als echte Fenster mit Live-Textur an den Wänden. Desktop-Icons hängen als kleine Screens wie Item-Frames an der Wand.
  - Ein neuer Screen belegt **1 × 1 Block**. Vergrößern geht **nur rechteckig**.
  - Die **Wand ist die Grenze**. **Blöcke ohne Hitbox** werden ignoriert und lassen sich überziehen: Blumen, Gras, Fackeln, Töpfe und alles Kleine, Durchlaufbare, das kein voller Block, keine Slab und keine Treppe ist.
- **Sichtbarkeit und Last:**
  - Geöffnete Fenster bleiben offen und sind aus der Ferne sichtbar.
  - Entfernte Fenster und Fenster außerhalb des Blickfelds werden gedrosselt, auf etwa 1 Bild pro Sekunde.
- **Vollbild:** Die Welt wird dabei nicht gerendert.
- **Pausieren:** Weltzeit und Animationen halten an. Ist die Welt sichtbar, wird nur bei Fensteränderungen neu gezeichnet.
- **Eingaben** werden korrekt an das echte Fenster weitergegeben.

## Zu entscheiden

| Nr. | Frage | Empfehlung bzw. Hinweise |
| --- | ----- | ------------------------ |
| T11 | Fensterauflösung und Größenaushandlung | logische Größe und Skala (`fractional-scale-v1`) unabhängig von der Blockgröße; Pixeldichte pro Block einstellbar (16 px reichen für Text nicht); übergroße Puffer einpassen |
| T12 | Verankerung und Layout-Datei | Anker = Blockposition + Seite + Rechteck in Blöcken. Was passiert, wenn die Wand nach einem Weltupdate fehlt (Featureliste, Frage 10)? |
| D6 | Inhalt und Auflösung beim Aufziehen; 1×1-Screen für Programme mit Mindestgröße | Designplan, Frage 6 |
| D9 | Wo erscheinen neue Fenster und Dialoge? Wie werden Menüs gezeigt, die über den Screen hinausragen? | Designplan, Frage 9. **Muss vor M3 entschieden sein** |
| D10 | Tastaturfokus und Fluchttaste | Designplan, Frage 10. Vorbilder: Waylandcraft `G`/`Alt+Q`, SimulaVR Super+Z |
| F10 | Zählen Glas, Türen, Slabs und Treppen als Wandfläche? | Featureliste, Frage 10 |

## Technische Grundlagen

Belegt in R7–R9 und R15.

### Puffer und Synchronisation

- **dmabuf:** über `wgpu_hal::vulkan::Device::texture_from_dmabuf_fd` (wgpu ≥ 30, unsafe, nur eine Ebene, Feature `VULKAN_EXTERNAL_MEMORY_DMA_BUF`), dann `create_texture_from_hal`.
- **shm:** nur geänderte Bereiche hochladen.
- **Explicit Sync:** über Smithay `drm_syncobj`.
- Wie Import und Sync auf NVIDIA laufen, steht unter M0, „Unbedingt beachten".

### Eingabe

- UV in Fenstergeometrie-Koordinaten (`xdg_surface.set_window_geometry`) umrechnen.
- Oberste (Sub-)Surface bestimmen.
- `wl_pointer.enter/motion/button/axis` senden.
- Den Zeiger zeichnet der Compositor selbst.
- Für die Freisteuerung gibt es `pointer_constraints` und `relative_pointer`.

### Pop-ups und Dialoge

- **Pop-ups:**
  - `xdg_popup` mit `xdg_positioner`: Das Anker-Rechteck ist relativ zum Eltern-Surface; die Anpassung läuft in der Reihenfolge Flip, Slide, Resize.
  - Die Constraint-Fläche legt der Compositor frei fest.
  - Pop-ups werden eigene Quads knapp vor der Wand; der Raycast testet sie zuerst.
  - Bei einem Grab löst ein Klick in die Welt `popup_done` aus.
- **Dialoge:**
  - `xdg_toplevel.set_parent` bildet die Hierarchie.
  - `xdg-dialog-v1` (`set_modal`) ist nur ein Hinweis; ob Smithay es hat, ist unsicher.

### Größe, Dekoration, Zwischenablage

- **Größe:**
  - `configure` ist nur ein Hinweis; bindend ist er nur bei maximized und fullscreen.
  - Programme mit Mindestgröße liefern größere Puffer.
  - Tiled-States und `xdg-decoration` (serverseitig) verhindern Schatten und Titelleisten.
- **Zwischenablage und Drag & Drop:** Smithay `selection`; Drag & Drop nutzt denselben Raycast. Für den Explorer (M4) tritt der Compositor selbst als Datenquelle auf.
- **IME:** `text-input-v3` und `input-method-v2`; das Vorschlagsfenster sitzt am Cursor-Rechteck in der Wandebene.

### XWayland und Wine

- **xwayland-satellite** macht aus X11-Fenstern Wayland-Fenster. Programme mit festen Bildschirmkoordinaten können dabei brechen.
- **Ohne satellite:** Override-Redirect-Menüs haben Root-Koordinaten und müssen dem Eltern-Fenster zugeordnet werden.
- **Wine und Proton** laufen über XWayland. **Minecraft selbst** auch (auf NVIDIA robuster).

### Drosselung, Vollbild, Spiele

- **Drosselung:**
  - Frame-Callbacks zurückhalten (wirkt bei GTK, Qt, Browser; mpv rendert dann gar nicht).
  - `fifo-v1`-Barrieren für verdeckte Surfaces früher lösen.
  - Spiele mit MAILBOX bremsen nicht.
- **Vollbild:** Direct Scanout auf die Primary Plane, die Welt steht still.
- **Kopiergeschützte Videos** werden vermutlich nicht schwarz, weil Linux nur Widevine L3 hat (M0, P14).

## Unbedingt beachten

- **Hierarchie zuerst:** Ein Programm besteht aus Toplevel, Subsurfaces, Pop-ups und Dialogen. Diese Hierarchie von Anfang an sauber modellieren (Vorbild: das Panel-Modell von Stardust mit Child-Surfaces, Geometrie relativ zum Eltern-Surface, `z_order` und Eingaberegionen; nur die Technik).
- **Raycast-Reihenfolge:** Pop-ups, dann Dialoge, dann Fenster, dann Welt. Flache Winkel abfangen.
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

## Offene Fragen an den Nutzer

- Designplan, Fragen 6, 9, 10.
- Featureliste, Frage 10.

## Referenzen

- [xdg-shell](https://wayland.app/protocols/xdg-shell), [fractional-scale-v1](https://wayland.app/protocols/fractional-scale-v1), [fifo-v1](https://wayland.app/protocols/fifo-v1)
- [Smithay `XwmHandler`](https://docs.rs/smithay/latest/smithay/xwayland/xwm/trait.XwmHandler.html), [niri und xwayland-satellite](https://github.com/niri-wm/niri/wiki/Xwayland)
- **Vorbilder, nur die Technik:**
  - Waylandcraft (GPL-3): `WorldPlane`, Tastaturfang
  - Stardust XR (GPL-2): Panel-Modell
  - SimulaVR (MIT): Leinwand pro Fenster mit Überstand für Pop-ups
  - Wayfire `view-3d.cpp` (MIT): Umkehrabbildung
