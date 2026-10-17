# M0: Entscheidungen und Prototypen

**Planungsstand:** grob, noch nicht bereit. Es fehlen die Entscheidungen T1–T4 und T19, eine Priorisierung der Prototypen und R32/R33.
**Größe laut Featureliste:** 1–3 Wochen, Risiko hoch.
**Voraussetzung:** Linux-Umzug ([linux-umzug.md](../linux-umzug.md)) mindestens bis einschließlich NVIDIA-Treiber und Rust-Werkzeugen.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M0 nötig ist. Globale Leitplanken (nur lesen, kein Mojang-Code, 0BSD, kein GPL-Code, der Nutzer liest keinen Code) stehen in [CLAUDE.md](../CLAUDE.md). Hintergründe und Quellen stehen in [recherche-ergebnisse.md](recherche-ergebnisse.md). Messwerte, Screenshots und Erkenntnisse kommen unten unter „Ergebnisse".

## Ziel

Beweisen, dass das Abbruchrisiko des Projekts lösbar ist: **echte Programmfenster als Live-Textur auf einer schrägen 3D-Fläche, mit der Maus bedienbar, auf der RTX 5060 Ti unter Wayland.** Dazu die Grundsatzentscheidungen festmachen.

Ergebnis von M0 sind Wegwerf-Prototypen, Messwerte und Entscheidungen, noch kein Produktcode. Trägt der Fenster-Prototyp nicht, kippt das Projekt.

## Kern: darauf kommt es an

1. **Fensterpuffer ohne Kopie in die 3D-Szene** (dmabuf nach wgpu, P2), **mit Explicit Sync auf NVIDIA** (P3). Scheitert das, greift der Rückfall von T1: Smithays GlesRenderer mit eigenem GL-Code.
2. **Eingabe zurück ins Fenster:** Raycast, dann UV, dann Surface-Koordinate, dann `wl_pointer` (P5). Dazu gehören Pop-ups und Menüs (P6), denn ohne sie ist kein echtes Programm bedienbar.
3. **Messbar geringe Last:** Im Leerlauf kein einziges Neuzeichnen (P12) und Drosselung entfernter Fenster (P11). „Kaum Last" ist ein Kernversprechen des Projekts.

## Entschieden

- **Entwicklung direkt unter Linux**, auf einer 500-GB-Partition der NVMe-SSD. Windows bleibt übergangsweise im Dual Boot. Unter Windows lauffähig zu sein ist kein Ziel.
- **Hardware:** RTX 5060 Ti (Blackwell) mit 16 GB VRAM.
- **Lizenz 0BSD.** Die `LICENSE`-Datei kommt mit dem ersten Code ins Repository. Waylandcraft (GPL-3), niri (GPL-3) und Stardust (GPL-2) dienen nur als Anschauung, ihr Code wird nicht übernommen.
- **Claude Code programmiert.** Der Nutzer prüft Verhalten, Screenshots und Messwerte, keinen Code.

## In M0 zu entscheiden

| Nr. | Frage | Empfehlung | Wird bestätigt durch |
| --- | ----- | ---------- | -------------------- |
| T1 | Sprache und Grafik-API | **Rust + wgpu (≥ 30)**; rohes Vulkan über wgpu-hal nur für dmabuf und Sync. Rückfall: Smithays GlesRenderer (GLES 3) | P2, P3 |
| T2 | Compositor-Bibliothek | **Smithay**, Git-Master auf einen geprüften Commit festgelegt. Rückfall: wlroots über FFI | P1–P6 |
| T3 | Prozessaufteilung | **ein Prozess**; die Welt als eigene Crate hinter einer Schnittstelle, damit sie später in einen eigenen Prozess wandern kann | – |
| T4 | Distribution | **Fedora KDE Plasma** mit NVIDIA 595 (offen) über RPM Fusion und Snapper. Die Umzugsanleitung geht schon davon aus | – |
| T10 | Bildtaktung | Leerlauf: 0 Bilder. Stille Kamera mit Animationen: 20 Hz. Bewegung: volle Rate. Vollbild: Direct Scanout | P11–P13 |
| T11 | Fenstergröße und Auflösung | logische Größe und Skala unabhängig von der Blockgröße; Pixeldichte pro Block einstellbar; übergroße Puffer einpassen | P7 |
| T19 | Sprache im Code | offen (Englisch ist bei Rust üblich, Deutsch wäre für den Nutzer lesbar) | Nutzer |

## Technischer Rahmen (gilt, sobald T1–T3 bestätigt sind)

- **Cargo-Workspace** mit diesen Crates:
  - `welt`: Formate, Meshing, Renderer; ohne Abhängigkeit vom Compositor
  - `viewer`: die Welt als normales Fenster, für M1 und M2
  - `compositor`: Smithay
  - `shell`: ab M4
- **Prototypen** liegen getrennt unter `prototypen/` und dürfen unsauber sein.
- **Nested-Entwicklung:** Der Compositor läuft über Smithays winit-Backend als Fenster in Plasma (Wayland).
  - Plasma fängt die Super-Taste ab. Deshalb ist die Modifier-Taste der Shell konfigurierbar und während der Entwicklung eine andere.
  - Das winit-Backend ist bei Hotkeys fehlerhaft (niri) und antwortet nested einen Frame zu spät (Smithay #2208).
- **Qualität:**
  - `cargo fmt` und `cargo clippy -- -D warnings` müssen sauber sein.
  - Tests gehören zu jedem Baustein.
  - `unsafe` bleibt auf wenige kleine Module beschränkt (dmabuf-Import, Vulkan-Interop), jedes mit einem Kommentar zur Sicherheitsbegründung.

## Unbedingt beachten

### NVIDIA

- **Treiber:** Nur die offenen Kernelmodule, Production-Zweig 595. Die Feature-Branches ab 610 bringen eine Color-Pipeline mit, die manche Compositors stört.
- **Explicit Sync ist Pflicht** (`linux-drm-syncobj-v1`), sonst gibt es Grafikfehler, weil NVIDIA kein implizites Sync kann.
  - Smithays `drm_syncobj` von Anfang an einbauen und nur anbieten, wenn `supports_syncobj_eventfd()` true liefert.
  - Der `DrmSyncPointBlocker` hält einen Commit zurück, bis der Acquire-Fence signalisiert ist. Vermutlich muss wgpu deshalb keine fremden Semaphoren importieren (unsicher, R33).
  - Den Release-Punkt erst signalisieren, wenn die GPU fertig gelesen hat, zum Beispiel nach `Queue::on_submitted_work_done`. In P3 prüfen.
- **dmabuf-Import:**
  - Nur Modifier anbieten, die der Treiber meldet (dmabuf-Feedback).
  - Den **Listen-Pfad** verwenden (`VkImageDrmFormatModifierListCreateInfoEXT`), nicht den expliziten Layout-Pfad: Der lehnt auf NVIDIA gültige Layouts ab.
  - Getilte Modifier können als Renderziel scheitern.
  - `texture_from_dmabuf_fd` kann nur Puffer mit einer Ebene. Mehrebenige Formate wie NV12-Video erst einmal per Kopie oder gar nicht.
  - DRM-fourcc → wgpu/Vulkan-Format selbst abbilden; ARGB und XRGB in Little-Endian beachten (Farbvertauschung, P1).
- **Bekannte Smithay-Probleme auf NVIDIA:** #1988 (VRAM-Erschöpfung, Clients erholen sich nicht) und #1862 (Flackern bei hoher Bildwiederholrate). VRAM-Verbrauch in Langläufen beobachten.

### Rendern und Eingabe

- **Smithay rendert nur GLES.** Unser Rendern läuft komplett selbst mit wgpu; Smithay liefert Protokolle, Puffer und später DRM. Nested ist das unproblematisch (eigene wgpu-Surface im winit-Fenster). Wie die Bilder direkt auf DRM kommen, ist offen (P15, später).
- **Pro Fenster eine Offscreen-Textur** mit dem ganzen Surface-Baum (Toplevel und Subsurfaces) und Mip-Kette. Daran hängen später Drosselung und Vorschau.
- **Raycast:**
  - Strahl mit der Ebene des Quads schneiden, UV berechnen, in Fenstergeometrie-Koordinaten (`set_window_geometry`) umrechnen.
  - `surface_at` für die oberste (Sub-)Surface, dann `wl_pointer.enter/motion/button/axis` mit surface-lokalen Koordinaten.
  - Bei sehr flachem Blickwinkel versagt die Umkehrabbildung (Determinante 0, wie bei Wayfire): abfangen, nicht abstürzen.
  - Scroll-Ereignisse vollständig weitergeben (bei Stardust fehlen welche).
- **Pop-ups:**
  - Die Fläche, an der sich `xdg_positioner` ausrichtet (Constraint-Fläche), legt der Compositor selbst fest, zum Beispiel das Fensterrechteck mit Rand.
  - Pop-ups als eigene Quads knapp vor der Wand in derselben Ebene zeichnen; der Raycast testet sie **vor** dem Fenster.
  - Ein Klick in die Welt schließt Pop-ups mit Grab (`popup_done`).
- **Fenstergröße:**
  - `configure` ist nur ein Hinweis; Programme mit Mindestgröße liefern größere Puffer, die eingepasst werden.
  - Tiled-States setzen und serverseitige Dekoration anfordern (`xdg-decoration`), damit keine Schatten und Titelleisten an der Wand landen.
- **Drosselung:**
  - Frame-Callbacks zurückhalten bremst GTK, Qt, Browser und mpv.
  - Spiele mit MAILBOX oder IMMEDIATE bremsen nicht; sie werden nur seltener abgetastet.
  - Die `fifo-v1`-Barriere für verdeckte Surfaces früher lösen.

### Platzhalter für offene Bedienfragen

Wo neue Fenster erscheinen, welcher Screen die Tastatur bekommt und wie man sie zurückholt, ist noch offen (Designplan, Fragen 9 und 10). Im Prototyp gelten Platzhalter:
- neues Fenster auf das nächste freie Test-Quad
- Fokus per Klick
- feste Fluchttaste

## Prototyp-Tests

Priorität: **K** = Kern, entscheidet über T1/T2 und den Fortgang. **W** = wichtig für M3. **S** = später, darf nach M3 oder M8 wandern.

| Nr. | Prio | Test | Erfolgskriterium |
| --- | ---- | ---- | ---------------- |
| P1 | W | shm-Clients (`weston-simple-shm`, foot) auf einem Quad | Inhalt und Farben korrekt, Update unter einem Frame Verzögerung |
| P2 | K | dmabuf ohne Kopie (Firefox, vkcube, `weston-simple-dmabuf-egl`) über `texture_from_dmabuf_fd` | korrektes Bild bei mindestens 2 Modifiern, keine CPU-Kopie im Profiler |
| P3 | K | Explicit Sync mit einem NVIDIA-Client | kein Flackern und kein Tearing bei vkcube mit 60 fps über 5 min |
| P4 | W | Mip-Kette und anisotrope Filterung, 1080p-Fenster unter 70° | Text in 2–3 Blöcken Abstand lesbar, kein Flimmern, unter 1 ms GPU pro Update |
| P5 | K | Raycast-Klick auf Schaltflächen am Rand eines GTK4- und eines Qt-Programms, auch schräg | pixelgenau (±1 px), Hover-Effekte stimmen |
| P6 | K | Pop-ups: Kontextmenü, Combobox, verschachteltes Menü, Tooltip (GTK4, Qt, Firefox) | richtige Position am Quad, Klick in die Welt schließt das Menü, keine Protokollfehler |
| P7 | W | kleine quadratische Größe (400×400) und große logische Größe mit Skala 2 | Programme bedienbar, übergroße Puffer sauber eingepasst |
| P8 | S | Zwischenablage zwischen Fenstern, Drag & Drop einer Datei in den Browser | Text und Datei kommen an |
| P9 | S | IME (fcitx5) in GTK und Qt | Vorschlagsfenster am Cursor im Quad |
| P10 | W | XWayland über `xwayland-satellite` (xeyes, ein Wine-Programm, Minecraft) | sichtbar, klickbar, Menüs am richtigen Ort |
| P11 | W | Drosselung auf 1 Hz (GTK, Firefox mit Video, mpv, vkcube FIFO und MAILBOX) | Last sinkt um mehr als 80 % (außer MAILBOX), kein Hänger beim Zurückschalten |
| P12 | K | Leerlauf ohne Eingabe und ohne Änderungen | 0 Redraws pro Sekunde |
| P13 | S | Vollbild-Spiel mit Direct Scanout | Weltrenderer steht still, fps wie unter Plasma |
| P14 | S | DRM-Video (Netflix in Firefox) als Wandtextur | feststellen, ob das Bild schwarz wird |
| P15 | S | wgpu rendert in einen Puffer, den Smithays DRM-Ausgabe anzeigt (eigenes TTY, nicht nested) | Bild auf dem Bildschirm ohne Kopie |
| P16 | W | eine echte Regionsdatei aus der Welt des Nutzers (26.x) lesen, einen Chunk im Worker-Thread zu einem Mesh bauen | fastanvil oder simdnbt liest das Format; Grundlage für T6 und T9 |
| P17 | S | RenderDoc-Mitschnitt des nested Renderers | Mitschnitt funktioniert |

## Abnahme von M0 (ohne Codelesen prüfbar)

- **Demo:** Im nested Compositor auf NVIDIA hängen ein Firefox-Fenster und ein GTK-Programm auf schrägen Flächen in einer einfachen 3D-Szene. Der Nutzer kann:
  - Schaltflächen klicken
  - ein Kontextmenü öffnen
  - Text tippen
  - scrollen
  Nachweis per Video oder Screenshots.
- **Messwerte** von P11 und P12 stehen unten unter „Ergebnisse".
- Alle **K-Tests** sind bestanden, oder ein dokumentierter Rückfall ist gewählt.
- **T1–T4, T10, T11 und T19** stehen in [implementierungsplan.md](../implementierungsplan.md) als entschieden, und die Meilenstein-Dateien sind angepasst.

## Offene Fragen an den Nutzer

- Designplan Frage 9: Wo erscheinen neue Fenster und Dialoge?
- Designplan Frage 10: Tastaturfokus und Fluchttaste.
- Designplan Frage 6: Fenstergröße auf kleinen Screens.

In M0 genügen Platzhalter; für M3 muss die Antwort feststehen.

## Referenzen

Technik anschauen, keinen Code übernehmen, außer bei MIT- oder Apache-Lizenz.

- **wgpu und Smithay:**
  - [wgpu-hal `texture_from_dmabuf_fd`](https://docs.rs/wgpu-hal/30.0.0/wgpu_hal/vulkan/struct.Device.html)
  - [Smithay](https://github.com/Smithay/smithay) (MIT; Beispiel-Compositor „anvil")
  - [Smithay `drm_syncobj`](https://docs.rs/smithay/latest/smithay/wayland/drm_syncobj/index.html)
  - [Smithay-Diskussion #431](https://github.com/Smithay/smithay/discussions/431) zu wgpu
- **Wayland-Protokolle:**
  - [xdg-shell](https://wayland.app/protocols/xdg-shell)
  - [fifo-v1](https://wayland.app/protocols/fifo-v1)
  - [fractional-scale-v1](https://wayland.app/protocols/fractional-scale-v1)
- **Vorbilder:**
  - [Wayfire `view-3d.cpp`](https://github.com/WayfireWM/wayfire) (MIT)
  - [niri Redraw Loop](https://niri-wm.github.io/niri/Development:-Redraw-Loop.html) (GPL, nur die Idee)
  - [Waylandcraft](https://github.com/EVV1E/waylandcraft) (GPL-3, nur die Technik)
  - [Stardust XR](https://github.com/StardustXR/server) (GPL-2, nur die Technik)

## Ergebnisse

Wird während M0 gefüllt: Messwerte, Screenshots, gewählte Rückfälle, Erkenntnisse für M3.
