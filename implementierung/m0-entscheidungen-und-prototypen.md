# M0: Entscheidungen und Prototypen

**Planungsstand:** bereit (18.10.2026). E1 ausgeführt, A1–A5 erledigt (18.10.2026). Offen: A7, A8; A6 verschoben.
**Größe:** 1–3 Wochen, Risiko hoch.
**Voraussetzung:** Linux-Umzug erledigt; Einrichtung E1.

## Ziel

Beweisen, dass das Abbruchrisiko des Projekts lösbar ist: **echte Programmfenster als Live-Textur auf einer schrägen 3D-Fläche, mit der Maus bedienbar, auf der RTX 5060 Ti unter Wayland.** Dazu die Grundsatzentscheidungen festmachen.

Ergebnis von M0 sind Wegwerf-Prototypen, Messwerte und Entscheidungen, noch kein Produktcode. Trägt der Fenster-Prototyp nicht, kippt das Projekt.

## Kern: darauf kommt es an

1. **Fensterpuffer ohne Kopie in die 3D-Szene** (dmabuf nach wgpu, P2), **mit Explicit Sync auf NVIDIA** (P3). Scheitert das, greift der Rückfall von T1.
2. **Eingabe zurück ins Fenster:** Raycast, dann UV, dann Surface-Koordinate, dann `wl_pointer` (P5). Dazu gehören Pop-ups und Menüs (P6), denn ohne sie ist kein echtes Programm bedienbar.

Performance (Last, Bildtaktung: A6, P11, P12, T10) ist aus M0 ausgeklammert → Performance-Durchgang ([implementierungsplan.md](../implementierungsplan.md) §3).

## Entscheidungen

Vom Nutzer bestätigt am 18.10.2026:

| Nr. | Frage | Entscheidung |
| --- | ----- | ------------ |
| T1 | Sprache und Grafik-API | **Rust + wgpu**; rohes Vulkan über wgpu-hal nur für dmabuf und Sync. Rückfall, falls P2/P3 scheitern: Smithays GlesRenderer (GLES 3) mit eigenem GL-Code |
| T2 | Compositor-Bibliothek | **Smithay**, Git-Stand auf einen geprüften Commit festgelegt (letztes Release 0.7.0 ist von 06/2025). Rückfall: wlroots über FFI |
| T3 | Prozessaufteilung | **ein Prozess**; die Welt als eigene Crate hinter einer Schnittstelle, damit sie später in einen eigenen Prozess wandern kann. Notfallmodus zunächst über eine zweite Sitzung (m8) |
| T4 | Distribution und Desktop | **Fedora Workstation mit GNOME** (installiert: Fedora 44, GNOME 50.6, Wayland), zugleich Host für die nested Entwicklung und Notfallsitzung in m8. Kein KDE Plasma. Snapper ist eingerichtet. Upgrade auf Fedora 45 später |
| – | NVIDIA-Treiber | **bei 615 bleiben** (installiert: offenes Modul 615.71.09 über RPM Fusion), kein Rückbau auf 595. Bei Fehlern in P2/P3 mit 595 gegenprüfen |
| T19 | Sprache im Code | **Bezeichner und Kommentare Englisch** (auch Testnamen und Crate-Namen); **Deutsch** sind Commit-Nachrichten, Doku und alle Texte, die der Nutzer sieht (Oberfläche, Meldungen, Testberichte zur Abnahme) |

Noch offen, werden durch Prototypen bestätigt:

| Nr. | Frage | Empfehlung | Bestätigt durch |
| --- | ----- | ---------- | --------------- |
| T10 | Bildtaktung | Leerlauf: 0 Bilder. Stille Kamera mit Animationen: 20 Hz. Bewegung: volle Rate. Vollbild: Direct Scanout | P11–P13, im Performance-Durchgang |
| T11 | Fenstergröße und Auflösung | logische Größe und Skala (`fractional-scale-v1`) unabhängig von der Blockgröße; Pixeldichte pro Block einstellbar (16 px reichen für Text nicht; Waylandcraft nimmt standardmäßig 500 px pro Block); übergroße Puffer einpassen | P4, P7 |

## Technischer Rahmen

- **Verzeichnis:** Prototypen als eigener Cargo-Workspace unter `prototypen/`, dürfen unsauber sein. `LICENSE` (0BSD) kommt mit dem ersten Commit. Crates des Produkts: [implementierungsplan.md](../implementierungsplan.md) §2.
- **Werkzeuge:** Rust aus den Fedora-Paketen (1.99, Smithay braucht ≥ 1.87).
- **wgpu:** Git-Stand auf einen Commit festgelegt, bis wgpu 31 erscheint. Grund: Den Besitzwechsel importierter Bilder zwischen Queue-Familien (`hal::TextureBarrier::queue_family_ownership_transfer`, PR #9668) gibt es erst nach 30.0.1.
- **Smithay:** `default-features = false`, Features `wayland_frontend` und `backend_drm` (für `DrmDeviceFd` und `drm_syncobj`), wie bei Waylandcraft.
- **Nested-Entwicklung:** **eigenes winit-Fenster mit wgpu-Surface**, nicht Smithays winit-Backend: Das verlangt einen `GlesRenderer`. Eingaben aus winit selbst an Smithays Seat weitergeben. Damit entfallen auch die Hotkey-Fehler (niri) und der verspätete Frame (Smithay #2208) dieses Backends.
- **GNOME als Host** fängt die Super-Taste ab; Modifier der Shell deshalb konfigurierbar (implementierungsplan.md §2).

## Unbedingt beachten

### NVIDIA

- **Treiber 615:** Die DRM-Color-Pipeline ab 610 lässt manche Compositors beim Einschalten von HDR schwarz werden; nur bei direkter DRM-Ausgabe relevant (P15, m8), nested nicht. Abhilfe: Modulparameter `nvidia-drm` `color_pipeline` abschalten.
- **Explicit Sync ist Pflicht** (`linux-drm-syncobj-v1`), sonst gibt es Grafikfehler, weil NVIDIA kein implizites Sync kann. (Waylandcraft bietet kein syncobj an und läuft laut README auf NVIDIA; wie gut, ist unklar.)
  - Smithays `drm_syncobj` von Anfang an einbauen und nur anbieten, wenn `supports_syncobj_eventfd()` true liefert. Den Render-Node der wgpu-GPU aus Vulkan bestimmen (`VkPhysicalDeviceDrmPropertiesEXT`) und als `DrmDeviceFd` öffnen; hier ist NVIDIA `renderD129`, `renderD128` ist die Intel-iGPU.
  - **Acquire:** Der `DrmSyncPointBlocker` hält den Commit per eventfd zurück, bis der Acquire-Punkt signalisiert ist. wgpu muss deshalb **keine fremden Semaphoren importieren** (kann es auch nicht).
  - **Release:** Smithay signalisiert den Release-Punkt, sobald alle Referenzen auf den Smithay-`Buffer` freigegeben sind. Den `Buffer` also halten, bis die GPU fertig gelesen hat (`Queue::on_submitted_work_done`), erst dann freigeben. In P3 prüfen.
- **dmabuf-Import** über wgpu-hal `texture_from_dmabuf_fd`, dann `create_texture_from_hal`:
  - Nur Modifier anbieten, die der Treiber meldet (dmabuf-Feedback).
  - wgpu nutzt den **expliziten Layout-Pfad** (`VkImageDrmFormatModifierExplicitCreateInfoEXT`). Der bekannte NVIDIA-Fehler dort betrifft nur mehrebenige Formate (NV12); für Puffer mit einer Ebene sollte er gehen (P2). Scheitert er, eigener Import mit dem Listen-Pfad (`VkImageDrmFormatModifierListCreateInfoEXT`) über `texture_from_raw`.
  - Vulkan übernimmt den fd: Smithays fd vorher duplizieren.
  - Import pro dmabuf einmal und zwischenspeichern; Clients wechseln zwischen 2–3 Puffern.
  - Vor dem Lesen das Bild von der fremden Queue-Familie übernehmen, danach zurückgeben. Umgesetzt mit `VK_QUEUE_FAMILY_EXTERNAL` per rohem `cmd_pipeline_barrier`: wgpu aktiviert `VK_EXT_queue_family_foreign` nicht, und wgpu-hals Barrieren-API kollidiert mit dem Snatch-Lock (Panik).
  - Die Textur als initialisiert anlegen (`create_texture_from_hal` mit `cleared`), sonst löscht wgpu sie.
  - Nur Puffer mit einer Ebene. Mehrebenige Formate wie NV12-Video erst einmal per Kopie oder gar nicht.
  - Getilte Modifier können als Renderziel scheitern.
  - DRM-fourcc → wgpu/Vulkan-Format selbst abbilden; ARGB und XRGB in Little-Endian beachten (Farbvertauschung, P1).
  - Vorbild: offener wgpu-PR #10522 (GLES-Import, Format-Abfrage, Besitzwechsel) aus einem Smithay-und-wgpu-Compositor.
- **Bekannte Smithay-Probleme auf NVIDIA:** #1988 (VRAM-Erschöpfung, Clients erholen sich nicht) und #1862 (Flackern bei hoher Bildwiederholrate). VRAM-Verbrauch in Langläufen beobachten.

### Rendern und Eingabe

- **Smithay rendert nur GLES.** Unser Rendern läuft komplett selbst mit wgpu; Smithay liefert Protokolle, Puffer und später DRM. Wie die Bilder direkt auf DRM kommen, ist offen (P15, später).
- **Pro Fenster eine Offscreen-Textur** mit dem ganzen Surface-Baum (Toplevel und Subsurfaces) und Mip-Kette. Daran hängen später Drosselung und Vorschau.
- **Raycast:**
  - Strahl mit der Ebene des Quads schneiden, UV berechnen, in Fenstergeometrie-Koordinaten (`set_window_geometry`) umrechnen.
  - `surface_at` für die oberste (Sub-)Surface, dann `wl_pointer.enter/motion/button/axis` mit surface-lokalen Koordinaten.
  - Bei sehr flachem Blickwinkel versagt die Umkehrabbildung (Determinante 0, wie bei Wayfire): abfangen, nicht abstürzen.
  - Scroll-Ereignisse vollständig weitergeben (bei Stardust fehlen welche).
- **Pop-ups:**
  - Die Fläche, an der sich `xdg_positioner` ausrichtet (Constraint-Fläche), legt der Compositor selbst fest, zum Beispiel das Fensterrechteck mit Rand. (Waylandcraft übernimmt die Positioner-Geometrie unverändert und ignoriert Grabs; das reicht nicht.)
  - Pop-ups als eigene Quads knapp vor der Wand in derselben Ebene zeichnen; der Raycast testet sie **vor** dem Fenster.
  - Ein Klick in die Welt schließt Pop-ups mit Grab (`popup_done`).
  - `xwayland-satellite` macht aus X11-Override-Redirect-Fenstern (Menüs) per Heuristik `xdg_popup`s (P10).
- **Dialoge:** `xdg-dialog-v1` ist in Smithay vorhanden (`shell::xdg::dialog`, `XdgDialogState`).
- **Fenstergröße:**
  - `configure` ist nur ein Hinweis; Programme mit Mindestgröße liefern größere Puffer, die eingepasst werden.
  - Tiled-States setzen und serverseitige Dekoration anfordern (`xdg-decoration`), damit keine Schatten und Titelleisten an der Wand landen.
- **Drosselung:**
  - Frame-Callbacks zurückhalten bremst GTK, Qt, Browser und mpv. Offen: Eine frühere Fassung von m3 sagte, mpv rendere dann gar nicht mehr; P11 klärt das.
  - Spiele mit MAILBOX oder IMMEDIATE bremsen nicht; sie werden nur seltener abgetastet.
  - Die `fifo-v1`-Barriere für verdeckte Surfaces früher lösen.

### Platzhalter für offene Bedienfragen

Wo neue Fenster erscheinen, welcher Screen die Tastatur bekommt und wie man sie zurückholt, ist offen (D9, D10). Im Prototyp gelten Platzhalter:
- neues Fenster auf das nächste freie Test-Quad
- Fokus per Klick
- feste Fluchttaste

## Einrichtung

**E1** (Nutzer, root, einmal im Terminal):

```bash
sudo dnf install rust cargo clippy rustfmt wayland-devel libxkbcommon-devel libinput-devel systemd-devel libseat-devel mesa-libgbm-devel libdrm-devel vulkan-tools vulkan-validation-layers weston foot xwayland-satellite mpv gtk4-devel-tools qt6-qtbase-examples xeyes renderdoc weston-demo
```

Testprogramme darin: `weston-simple-shm`, `weston-simple-dmabuf-egl` (weston-demo, nicht im Paket weston), foot, `vkcube` (vulkan-tools), `gtk4-demo` (gtk4-devel-tools), Qt-Beispiele, mpv, `xeyes`, RenderDoc. Firefox ist vorinstalliert. Abnahme: `cargo --version` und `vkcube` laufen.

## Teilaufgaben

Reihenfolge = Priorität. Nach jeder Teilaufgabe Ergebnis unten unter „Ergebnisse" eintragen.

| Nr. | Inhalt | Tests | Abnahme (ohne Codelesen) |
| --- | ------ | ----- | ------------------------ |
| A1 | Workspace `prototypen/`, `LICENSE`, winit-Fenster mit wgpu, frei bewegliche Kamera, zwei schräge Test-Quads | – | Screenshot der 3D-Szene; `cargo clippy` sauber |
| A2 | Smithay-Display im selben Prozess, Socket, xdg-shell, shm-Puffer auf Quad | P1 | foot und `weston-simple-shm` mit korrekten Farben auf einem schrägen Quad (Screenshot) |
| A3 | dmabuf-Import ohne Kopie, dmabuf-Feedback, `drm_syncobj` | P2, P3 | Firefox und vkcube auf Quads; vkcube 5 min ohne Flackern (Video); Log nennt Modifier und Sync-Weg |
| A4 | Raycast-Eingabe: Zeiger, Klick, Scrollen, Tastatur mit Fokus per Klick und Fluchttaste | P5 | GTK4- und Qt-Programm schräg bedienbar, Randknöpfe treffen, Text tippen geht |
| A5 | Pop-ups mit Grab und Constraint-Fläche | P6 | Kontextmenü, Combobox, verschachteltes Menü, Tooltip in GTK4, Qt und Firefox (Screenshots) |
| A6 | Bildtaktung: Leerlauf, Drosselung. **Verschoben** in den ersten Performance-Durchgang | P12, P11 | Messwerte Redraws/s und CPU/GPU-Last vorher/nachher |
| A7 | Demo für die Abnahme von M0 | – | siehe „Abnahme von M0" |
| A8 | W-Tests, soweit Zeit bleibt | P4, P7, P10, P16 | je Erfolgskriterium der Tabelle; Rest wandert nach M3 |

S-Tests wandern nach M3 bzw. M8 (P15).

## Prototyp-Tests

Priorität: **K** = Kern, entscheidet über den Fortgang oder den Rückfall von T1/T2. **W** = wichtig für M3. **S** = später, darf nach M3 oder M8 wandern.

| Nr. | Prio | Test | Erfolgskriterium |
| --- | ---- | ---- | ---------------- |
| P1 | W | shm-Clients (`weston-simple-shm`, foot) auf einem Quad | Inhalt und Farben korrekt, Update unter einem Frame Verzögerung |
| P2 | K | dmabuf ohne Kopie (Firefox, vkcube, `weston-simple-dmabuf-egl`) über `texture_from_dmabuf_fd` | korrektes Bild bei mindestens 2 Modifiern |
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
| P13 | S | Vollbild-Spiel mit Direct Scanout | Weltrenderer steht still, fps wie unter GNOME |
| P14 | S | DRM-Video (Netflix in Firefox) als Wandtextur | feststellen, ob das Bild schwarz wird |
| P15 | S | wgpu rendert in einen Puffer, den Smithays DRM-Ausgabe anzeigt (erst VM, dann eigenes TTY; m8 S3, S5) | Bild auf dem Bildschirm ohne Kopie |
| P16 | W | eine echte Regionsdatei aus der Welt des Nutzers (26.x) lesen, einen Chunk im Worker-Thread zu einem Mesh bauen | fastanvil oder simdnbt liest das Format; Grundlage für T6 und T9 |
| P17 | S | RenderDoc-Mitschnitt des nested Renderers | Mitschnitt funktioniert |

## Abnahme von M0 (ohne Codelesen prüfbar)

- **Demo:** Im nested Compositor auf NVIDIA hängen ein Firefox-Fenster und ein GTK-Programm auf schrägen Flächen in einer einfachen 3D-Szene. Der Nutzer kann:
  - Schaltflächen klicken
  - ein Kontextmenü öffnen
  - Text tippen
  - scrollen
  Nachweis per Video oder Screenshots.
- Alle **K-Tests** außer P12 (verschoben) sind bestanden, oder ein dokumentierter Rückfall ist gewählt.
- **T11** ist entschieden, sein Status im T-Verzeichnis ([implementierungsplan.md](../implementierungsplan.md)) ist aktualisiert.

## Offene Fragen an den Nutzer

D6, D9, D10 ([bedienung.md](../bedienung.md)): In M0 genügen Platzhalter, für M3 müssen sie feststehen.

## Referenzen

- **wgpu und Smithay:**
  - [wgpu-hal `texture_from_dmabuf_fd`](https://docs.rs/wgpu-hal/30.0.0/wgpu_hal/vulkan/struct.Device.html)
  - [wgpu PR #10522](https://github.com/gfx-rs/wgpu/pull/10522) (dmabuf-Import für einen Smithay-Compositor, offen)
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

Start: `cargo run -- --spawn foot` in `prototypen/`; Optionen: `cargo run -- --help`. Screenshots unter `assets/m0/`.

### A1 (erledigt 18.10.2026)

- Workspace `prototypen/`, eine Crate `nested`; `LICENSE` (0BSD) im Repo-Root.
- Feste Stände: wgpu Git `efcf445784dd` (enthält `queue_family_ownership_transfer`), Smithay Git `023b78b1c5a0` (Features wie „Technischer Rahmen"), winit 0.30.13. Abhängigkeiten in Debug-Builds mit `opt-level = 2`.
- GPU: RTX 5060 Ti über Vulkan, Treiber 615.71.09; Surface-Format `Rgba8UnormSrgb`, Present `AutoVsync`.
- Kamera frei beweglich, Steuerung in `--help`. Maus wird per Locked gefangen, sonst Confined.
- Gezeichnet wird nur auf Ereignis (`ControlFlow::Wait`), Basis für P12; nicht gemessen.
- Screenshot: `assets/m0/a1-szene.png`.

### A2 (erledigt 18.10.2026), P1 teilweise

- Smithay im selben Prozess. Socket `wayland-N`, erster freie ab 1. Ein Hilfsthread pollt Display- und Socket-fd und weckt winit per `EventLoopProxy`; kein calloop.
- Globals: compositor, xdg-shell, xdg-decoration (immer serverseitig), shm, wl_seat (Tastatur + Zeiger, Eingabe kommt mit A4), wl_output, data-device. foot startet ohne `wl_data_device_manager` nicht.
- shm-Formate: ARGB/XRGB8888 → `Bgra8UnormSrgb`, ABGR/XBGR8888 → `Rgba8UnormSrgb`. Alpha wird ignoriert (Inhalt vor Schwarz).
- Platzhalter D9: neues Fenster auf nächstes freies Quad, `configure` 1024×640, Tiled-States auf allen Seiten. Quad übernimmt das Seitenverhältnis des Puffers.
- Upload: ganzer Puffer per `write_texture` bei jedem Commit; Frame-Callbacks nach jedem gezeichneten Bild.
- Farben korrekt: Truecolor-Orange (255,128,0) aus `prototypen/scripts/color-test.sh` erscheint orange. Screenshots: `assets/m0/a2-foot-farbtest.png`, `assets/m0/a2-foot-simple-shm.png` (beide Clients gleichzeitig, schräge Quads).
- **P1 offen:** „Update unter einem Frame Verzögerung" nicht gemessen.
- **Für M3:** nur Damage hochladen; Subsurfaces und Pop-ups zeichnen (bisher nur die Root-Surface); Alpha-Blending.
- Transitive Abhängigkeiten unter BSD-2/BSD-3, Zlib, ISC (u. a. tiny-skia über winits Fensterdeko): erlaubt nach T17.

### A3 (erledigt 18.10.2026): P2 und P3 bestanden

- dmabuf ohne Kopie: `nested/src/dmabuf.rs`. 28 Format/Modifier-Paare (AR24, XR24, AB24, XB24; nur eine Ebene, importierbar laut Vulkan). Feedback mit dem Render-Node der wgpu-GPU.
- Importiert und korrekt angezeigt: Modifier `0x0` (linear, `weston-simple-dmabuf-egl`), `0x300000000606014` (vkcube, Firefox), `0x300000000606010` (Firefox). UNORM-Textur, sRGB-Dekodierung im Shader.
- Explicit Sync: `linux-drm-syncobj-v1` angeboten (Kernel kann `syncobj_eventfd`). vkcube und Firefox nutzen es (Log: „Sync-Weg explizit“). Acquire über `DrmSyncPointBlocker` in calloop; Release: Smithay-`Buffer` werden bis `on_submitted_work_done` gehalten.
- Ereignisschleife: calloop für Smithay; ein Hilfsthread pollt dessen fd und weckt winit.
- P3: vkcube 5 min, durchgehend etwa 60 Bilder/s, keine Fehler. Flackern visuell nicht per Video geprüft.
- **Pro Fenster eine Offscreen-Textur** mit dem ganzen Surface-Baum (`nested/src/window.rs`), Alpha vormultipliziert, Zuschnitt auf die Fenstergeometrie, Viewporter angeboten. Nötig für Firefox (Inhalt in Subsurface). Mip-Kette fehlt noch (P4).
- **Verdecktes Fenster:** winit liefert unter Wayland `RedrawRequested` erst nach dem Frame-Callback des Hosts; GNOME schickt keinen an verdeckte Fenster. Ohne Gegenmittel stehen alle Clients. Ersatztakt: Frame-Callbacks nach 50 ms ohne Host-Bild.
- Screenshots: `assets/m0/a3-firefox-vkcube.png`, `assets/m0/a3-vkcube-5min.png`.

### A4 (erledigt 18.10.2026): P5 bestanden

- Raycast Kamera → Quad-Ebene → UV → Fensterpixel → oberste Surface; Strahl fast parallel zur Ebene wird übersprungen.
- Zeiger, Klicks, Mausrad (v120), Tastatur (evdev-Code + 8, Layout aus GNOME `input-sources`, hier `de`), Cursor-Form der Clients (benannte Cursor).
- Platzhalter: Klick auf Screen = Tastaturfokus; Klick in die Welt = Kamera (Maus gefangen); Fluchttaste **Strg+Esc** (von GNOME nicht belegt).
- Genauigkeit über das Testskript: Abweichung ≤ 0,01 px bei allen Zielen, auch 9 px kleine Pfeile am Rand.
- GTK4 (`gtk4-widget-factory`) und Qt (`gallery`) schräg bedienbar: Text tippen, Kästchen, Zahlenfeld, Scrollen. Screenshots `assets/m0/a4-*.png`.
- **Testskript** für Abnahmen ohne Hand an der Maus: `--script DATEI`, Format im Kopf von `nested/src/script.rs`, Beispiele `prototypen/scripts/a4-input.txt`, `a5-popups.txt`. Echte Eingaben werden während des Skripts ignoriert.

### A5 (erledigt 18.10.2026): P6 teilweise

- Pop-ups als eigene Quads in der Ebene ihres Screens, je Ebene 2 mm davor; der Raycast testet sie vor dem Fenster.
- Constraint-Fläche: Fensterrechteck plus 32 px Rand (`POPUP_MARGIN`). `reposition` umgesetzt.
- Grab: Klick außerhalb schließt alle Pop-ups (`popup_done`, innerstes zuerst) und wird nicht weitergegeben; Tastaturfokus geht an das Pop-up und danach zurück.
- Bestanden: GTK-Kontextmenü an der richtigen Stelle, Schließen per Klick in die Welt; Qt-Combobox öffnet sich richtig, Auswahl per Tastatur wirkt. Screenshots `assets/m0/a5-*.png`.
- **Offen (für M3):** verschachteltes Menü, Tooltip, Firefox-Menüs nicht getestet. Smithay loggt „surface missing from known popups“ (Grab-Anfrage für ein Pop-up, das Smithay nicht mehr kennt); kein Protokollfehler an den Client, Ursache ungeklärt.

### Für M8 Teil 1 (P15)

- wgpu-hal bietet `Instance::create_surface_from_drm` (`VK_EXT_acquire_drm_display`): mögliche direkte Ausgabe auf DRM ohne GLES.

