# M0: Entscheidungen und Prototypen

**Planungsstand:** bereit (18.10.2026). Vor A1 muss der Nutzer den Einrichtungsbefehl E1 ausführen.
**Größe:** 1–3 Wochen, Risiko hoch.
**Voraussetzung:** Linux-Umzug erledigt; Einrichtung E1.

## Ziel

Beweisen, dass das Abbruchrisiko des Projekts lösbar ist: **echte Programmfenster als Live-Textur auf einer schrägen 3D-Fläche, mit der Maus bedienbar, auf der RTX 5060 Ti unter Wayland.** Dazu die Grundsatzentscheidungen festmachen.

Ergebnis von M0 sind Wegwerf-Prototypen, Messwerte und Entscheidungen, noch kein Produktcode. Trägt der Fenster-Prototyp nicht, kippt das Projekt.

## Kern: darauf kommt es an

1. **Fensterpuffer ohne Kopie in die 3D-Szene** (dmabuf nach wgpu, P2), **mit Explicit Sync auf NVIDIA** (P3). Scheitert das, greift der Rückfall von T1.
2. **Eingabe zurück ins Fenster:** Raycast, dann UV, dann Surface-Koordinate, dann `wl_pointer` (P5). Dazu gehören Pop-ups und Menüs (P6), denn ohne sie ist kein echtes Programm bedienbar.
3. **Messbar geringe Last:** Im Leerlauf kein einziges Neuzeichnen (P12) und Drosselung entfernter Fenster (P11). „Kaum Last" ist ein Kernversprechen des Projekts.

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
| T10 | Bildtaktung | Leerlauf: 0 Bilder. Stille Kamera mit Animationen: 20 Hz. Bewegung: volle Rate. Vollbild: Direct Scanout | P11–P13 |
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
  - Smithays `drm_syncobj` von Anfang an einbauen und nur anbieten, wenn `supports_syncobj_eventfd()` true liefert. Nested den Render-Node (`/dev/dri/renderD128`) selbst als `DrmDeviceFd` öffnen.
  - **Acquire:** Der `DrmSyncPointBlocker` hält den Commit per eventfd zurück, bis der Acquire-Punkt signalisiert ist. wgpu muss deshalb **keine fremden Semaphoren importieren** (kann es auch nicht).
  - **Release:** Smithay signalisiert den Release-Punkt, sobald alle Referenzen auf den Smithay-`Buffer` freigegeben sind. Den `Buffer` also halten, bis die GPU fertig gelesen hat (`Queue::on_submitted_work_done`), erst dann freigeben. In P3 prüfen.
- **dmabuf-Import** über wgpu-hal `texture_from_dmabuf_fd`, dann `create_texture_from_hal`:
  - Nur Modifier anbieten, die der Treiber meldet (dmabuf-Feedback).
  - wgpu nutzt den **expliziten Layout-Pfad** (`VkImageDrmFormatModifierExplicitCreateInfoEXT`). Der bekannte NVIDIA-Fehler dort betrifft nur mehrebenige Formate (NV12); für Puffer mit einer Ebene sollte er gehen (P2). Scheitert er, eigener Import mit dem Listen-Pfad (`VkImageDrmFormatModifierListCreateInfoEXT`) über `texture_from_raw`.
  - Vulkan übernimmt den fd: Smithays fd vorher duplizieren.
  - Import pro dmabuf einmal und zwischenspeichern; Clients wechseln zwischen 2–3 Puffern.
  - Vor dem Lesen das Bild von der fremden Queue-Familie übernehmen (`QueueFamily::Foreign`), danach zurückgeben.
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
sudo dnf install rust cargo clippy rustfmt wayland-devel libxkbcommon-devel libinput-devel systemd-devel libseat-devel mesa-libgbm-devel libdrm-devel vulkan-tools vulkan-validation-layers weston foot xwayland-satellite mpv gtk4-devel-tools qt6-qtbase-examples xorg-x11-apps renderdoc
```

Testprogramme darin: `weston-simple-shm`, `weston-simple-dmabuf-egl` (weston), foot, `vkcube` (vulkan-tools), `gtk4-demo` (gtk4-devel-tools), Qt-Beispiele, mpv, `xeyes` (xorg-x11-apps), RenderDoc. Firefox ist vorinstalliert. Abnahme: `cargo --version` und `vkcube` laufen.

## Teilaufgaben

Reihenfolge = Priorität. Nach jeder Teilaufgabe Ergebnis unten unter „Ergebnisse" eintragen.

| Nr. | Inhalt | Tests | Abnahme (ohne Codelesen) |
| --- | ------ | ----- | ------------------------ |
| A1 | Workspace `prototypen/`, `LICENSE`, winit-Fenster mit wgpu, frei bewegliche Kamera, zwei schräge Test-Quads | – | Screenshot der 3D-Szene; `cargo clippy` sauber |
| A2 | Smithay-Display im selben Prozess, Socket, xdg-shell, shm-Puffer auf Quad | P1 | foot und `weston-simple-shm` mit korrekten Farben auf einem schrägen Quad (Screenshot) |
| A3 | dmabuf-Import ohne Kopie, dmabuf-Feedback, `drm_syncobj` | P2, P3 | Firefox und vkcube auf Quads; vkcube 5 min ohne Flackern (Video); Log nennt Modifier und Sync-Weg |
| A4 | Raycast-Eingabe: Zeiger, Klick, Scrollen, Tastatur mit Fokus per Klick und Fluchttaste | P5 | GTK4- und Qt-Programm schräg bedienbar, Randknöpfe treffen, Text tippen geht |
| A5 | Pop-ups mit Grab und Constraint-Fläche | P6 | Kontextmenü, Combobox, verschachteltes Menü, Tooltip in GTK4, Qt und Firefox (Screenshots) |
| A6 | Bildtaktung: Leerlauf, Drosselung | P12, P11 | Messwerte Redraws/s und CPU/GPU-Last vorher/nachher |
| A7 | Demo für die Abnahme von M0 | – | siehe „Abnahme von M0" |
| A8 | W-Tests, soweit Zeit bleibt | P4, P7, P10, P16 | je Erfolgskriterium der Tabelle; Rest wandert nach M3 |

S-Tests wandern nach M3 bzw. M8 (P15).

## Prototyp-Tests

Priorität: **K** = Kern, entscheidet über den Fortgang oder den Rückfall von T1/T2. **W** = wichtig für M3. **S** = später, darf nach M3 oder M8 wandern.

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
| P13 | S | Vollbild-Spiel mit Direct Scanout | Weltrenderer steht still, fps wie unter GNOME |
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
- **T10 und T11** sind entschieden, ihr Status im T-Verzeichnis ([implementierungsplan.md](../implementierungsplan.md)) ist aktualisiert.

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
