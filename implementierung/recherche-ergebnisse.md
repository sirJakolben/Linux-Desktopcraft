# Abgeschlossene Recherchen

Stand: 16.10.2026

Archiv mit Quellen; für die Umsetzung gelten die Meilenstein-Dateien. „(unsicher)" = nicht belegt, im Prototyp oder per Recherche prüfen. Die Kernaussagen zu wgpu (R7/R13) und Waylandcraft (R3) sind an der Quelle nachgeprüft.

## R1–R4: Vorbilder

**Fragen:**
- Gibt es Minecraft-Shells?
- Wie bringen 3D-Compositors und Minecraft-Mods Fenster als Textur in eine 3D-Szene und Eingaben zurück ins Fenster?

### Keine Minecraft-Shell, aber Waylandcraft

**Eine Minecraft-Shell gibt es nicht.** Am nächsten kommt **Waylandcraft** ([GitHub](https://github.com/EVV1E/waylandcraft); [Pressebericht](https://www.gamingonlinux.com/2026/05/you-think-youve-seen-it-all-and-then-theres-a-wayland-compositor-inside-minecraft-on-linux/)). Es geht die umgekehrte Richtung: ein Wayland-Compositor *in* Minecraft.

- **Rahmendaten:** GPL-3.0, etwa 3.000 Sterne, aktiv, MC 26.1.2. Rust-Kern, Fabric-Mod.
- **Puffer:** dmabuf wird per EGL ohne Kopie importiert (`EGL_EXT_image_dma_buf_import`).
- **Klicks:** Fenster liegen als Ebene in der Welt (`WorldPlane`). Ein Klick wird über den Schnittpunkt von Strahl und Ebene zugeordnet, die Auflösung ist als „Pixels per Block" einstellbar.
- **Tastaturfang:** `G`, harter Fang mit `Alt+Q`.
- **Probleme:** Auf NVIDIA ist `__GL_THREADED_OPTIMIZATIONS=0` nötig. Der Zink-Treiber macht Ärger. Mit Iris-Shadern bekommen Fenster breite Ränder. Text ist auf Distanz schlecht lesbar.

**Weitere Mods mit Desktop-Funktion:**
- Filecraft, CDM ChaosCrafts Devices, Dockercraft (tot)
- WinCraft, MonoView (Bildschirm spiegeln, nur Windows)
- MapVNC, CraftVM

### 3D-Compositors

- **xrdesktop** (Collabora, MIT, nur noch Wartung):
  - spiegelt Fenster aus KWin oder GNOME in VR
  - reicht Texturen über `GL_EXT_memory_object` an Vulkan weiter
  - Eingaben werden synthetisch erzeugt (libinputsynth), nicht im Compositor verteilt
  - lief zeitweise nicht auf Intel
  - [Repo](https://gitlab.freedesktop.org/xrdesktop/xrdesktop), [Collabora](https://collabora.com/news-and-blog/news-and-events/moving-the-linux-desktop-to-another-reality.html)
- **wxrd** (wlroots 0.15 plus eigener Vulkan-Renderer, MIT, inaktiv seit 2023):
  - dmabuf direkt nach Vulkan, shm per `memcpy` (nur der geänderte Bereich)
  - Pop-ups als eigenes 3D-Fenster mit Versatz
  - lief nicht auf NVIDIA
  - [Repo](https://gitlab.freedesktop.org/xrdesktop/wxrd)
- **SimulaVR** (Haskell auf einem Godot-3-Fork plus wlroots-Fork, MIT, aktiv):
  - Jedes Fenster bekommt eine eigene Leinwand mit Toplevel, Subsurfaces und Pop-ups, vergrößert um einen Überstand für Pop-ups.
  - Klick: Treffer auf der Leinwand, dann `surface_at`, dann `wl_seat`-Ereignisse.
  - Fokus folgt dem Blick, Super+Z gibt Maus und Tastatur frei.
  - [Repo](https://github.com/SimulaVR/Simula)
- **Stardust XR** (Rust, GPL-2.0, sehr aktiv):
  - Bevy 0.16 mit geforkten wgpu-, Bevy- und vulkano-Zweigen
  - Wayland läuft seit 03/2026 in einem eigenen Prozess (`wayland-service`)
  - dmabuf über vulkano mit Explicit Sync
  - Fenstermodell: Toplevel plus Child-Surfaces mit Geometrie relativ zum Eltern-Surface, `z_order` und Eingaberegionen
  - Raycast: Strahl schneidet die Ebene z=0, Pixel mit Y nach unten
  - [Repo](https://github.com/StardustXR/server)
- **Wayfire** (C++, wlroots, MIT):
  - `view-3d.cpp` rechnet mit einem 2×2-Gleichungssystem vom Bildschirm zurück auf die Fensterkoordinate. Steht das Fenster hochkant zum Betrachter, ist die Determinante 0.
  - Pop-ups hängen im selben Transformationsknoten.
  - Das Würfel-Plugin fängt alle Eingaben ab.
  - [Repo](https://github.com/WayfireWM/wayfire)
- **KWin „VR Mode"** ([MR 8671](https://invent.kde.org/plasma/kwin/-/merge_requests/8671), Entwurf): Eingabe und Fokus sind laut Autor noch neu zu bauen.

### Ältere 3D-Desktops

- **Project Looking Glass** und **Compiz** scheiterten unter X11 daran, Eingaben auf transformierte Fenster zurückzuführen.
  - Looking Glass brauchte einen gepatchten X-Server.
  - Der Input-Redirect-Patch für Compiz wurde nie übernommen; der Würfel war deshalb nicht bedienbar.
  - Unter Wayland entfällt das Problem, weil der Compositor die Eingaben selbst verteilt.
  - [xorg 2007](https://lists.freedesktop.org/archives/xorg/2007-June/024971.html)
- **BumpTop** war ein Physik-Schreibtisch für Dateien (Apache 2.0) und ist eher als Ideengeber für den Explorer interessant.

### Mods im Spiel

- **MCEF:** Offscreen-Chromium, jedes Frame ein CPU-Upload der geänderten Bereiche, Eingaben in Pixelkoordinaten. [Repo](https://github.com/CinemaMod/mcef)
- **WebDisplays:** Auflösung `Blöcke × 16 − 4` px, Klicks als Vektor auf der Blockseite. [Repo](https://github.com/montoyo/webdisplays)
- **CC:Tweaked:** zeigt nur ein Zeichenraster.

**Lehren:**
- Pixeldichte pro Block einstellbar machen; 16 px reichen für Text nicht.
- Mipmaps verwenden.
- Das Eingabemodell (Tastaturfang, Fluchttaste) früh festlegen.
- Nicht wie xrdesktop Eingaben synthetisieren.

## R5, R6, R14: Linux-Unterbau

**Fragen:**
- Wie baut man eine eigene Shell?
- Welche Compositor-Bibliothek?
- Wie bindet man einen eigenen Renderer ein, und wie teilt man die Prozesse auf?

### X11 und Grundbausteine

- **X11 ist keine Option mehr.** Plasma 6.8 läuft seit 10/2026 nur noch unter Wayland ([Phoronix](https://www.phoronix.com/news/KDE-Plasma-68-Wayland-Exclusive)), GNOME 50 ebenfalls (unsicher).
- **Bausteine eines Compositors:**
  - DRM/KMS für die Bildausgabe, GBM als Allocator
  - EGL oder Vulkan für den Puffer-Import
  - libinput und udev für die Eingabegeräte
  - libseat über logind oder seatd für den Gerätezugriff
  - Smithay hat für jeden Baustein ein Modul ([docs](https://docs.rs/smithay/latest/smithay/backend/index.html)).
- **Sitzung:**
  - Eine `.desktop`-Datei in `/usr/share/wayland-sessions/`.
  - systemd-User-Units, Vorbild niri mit `niri.service`, `niri-shutdown.target` und `niri --session`, das die Umgebung an systemd und D-Bus weitergibt ([niri-Wiki](https://github.com/YaLTeR/niri/wiki/Getting-Started)).
  - Alternativ uwsm.
- **Portale:**
  - Eigene `<desktop>-portals.conf` ([Doku](https://flatpak.github.io/xdg-desktop-portal/docs/portals.conf.html)).
  - Dateidialoge über `xdg-desktop-portal-gtk`.
  - Screenshot und Bildschirmfreigabe brauchen Capture-Protokolle im Compositor (`xdg-desktop-portal-wlr`) oder eine Nachbildung der Mutter-API.

### Compositor-Bibliotheken

- **Smithay** (Rust, MIT):
  - letztes Release 0.7.0 vom 24.06.2025; niri und cosmic-comp nutzen den Git-Stand
  - Protokolle: xdg-shell, layer-shell, XWayland, dmabuf, drm-syncobj, fractional-scale, fifo, commit-timing, text-input
  - nested über das winit-Backend (laut niri bei Hotkeys fehlerhaft) oder das X11-Backend
  - Renderer nur GLES2, Glow, Pixman und Multi-GPU; **kein Vulkan-Renderer**, Vulkan nur als Allocator
  - [Repo](https://github.com/Smithay/smithay), [Renderer](https://docs.rs/smithay/latest/smithay/backend/renderer/index.html)
- **wlroots** (C, MIT, 0.20 vom 03/2026):
  - größte Protokollabdeckung, eigener Vulkan-Renderer
  - die API bricht mit jedem Release; aus Rust nur über FFI nutzbar
- **Scheiden aus:**
  - Louvre (C++, LGPL, praktisch ein einzelner Entwickler, XWayland nur „rootful")
  - Mir (für Kiosk-Shells gedacht)
  - Aquamarine (nur der Backend-Teil eines Compositors)
  - KWin-Effekt (ABI bricht mit jedem Release, keine volle Kontrolle über Eingaben)

### Eigener Renderer

- Die Smithay-Maintainer halten wgpu als *internen* Smithay-Renderer für zu abstrakt. Ihre Empfehlung: Smithay übernimmt Protokolle, DRM und Puffer, der Compositor rendert die Client-Puffer selbst, etwa mit wgpu ([Diskussion #431](https://github.com/Smithay/smithay/discussions/431)).
- **Vorbilder:**
  - Nourish (Smithay, wgpu und Iced, 06/2026, sehr jung; [Phoronix](https://www.phoronix.com/news/Nourish-Wayland-Compositor))
  - gamescope (Vulkan, ein Prozess)
  - wxrd

### Absturz und Prozessaufteilung

- Stürzt der Compositor ab, sterben alle Programme mit.
  - Nur Qt 6.6 und neuer kann sich wieder verbinden (`QT_WAYLAND_RECONNECT=1`, Plasma setzt das automatisch).
  - `wl-restart` hält den Socket offen; das unterstützen KWin und Hyprland.
  - Für GTK gibt es nur einen Entwurfs-MR.
  - Einen allgemeinen Weg gibt es 2026 nicht ([Phoronix](https://www.phoronix.com/news/Qt-Wayland-Compositor-Restart), [wl-restart](https://github.com/ferdi265/wl-restart)).
- **Aufteilung in Prozesse:**
  - river 0.4 lagert den Fenstermanager über ein Protokoll in einen eigenen Prozess aus ([Blog](https://isaacfreund.com/blog/river-window-management/)).
  - Einen Weltrenderer in einem eigenen Prozess, der die dmabufs der Programme bekommt, hat noch niemand gebaut.
  - Möglich wäre ein privates Protokoll mit dmabufs und Fences, oder `ext-image-copy-capture` (mit einer Kopie).

## R7, R8, R9, R15: Fenster im eigenen Compositor

**Fragen:**
- Wie kommen Fensterinhalte als Textur in die Szene?
- Wie kommen Eingaben zurück ins Fenster?
- Wie laufen Pop-ups, XWayland und die Bildtaktung?

### Puffer und Synchronisation

- **Puffertypen:**
  - `wl_shm`: CPU-Speicher, eine Kopie beim Upload.
  - `linux-dmabuf-v1`: GPU-Speicher mit fd, fourcc und Modifier, Import ohne Kopie.
- **Import in Vulkan:**
  - Extensions `VK_EXT_external_memory_dma_buf` und `VK_EXT_image_drm_format_modifier`.
  - Den DRM-fourcc muss man selbst auf VkFormat umrechnen ([Khronos](https://registry.khronos.org/VulkanSC/specs/1.0-extensions/man/html/VK_EXT_image_drm_format_modifier.html)).
- **Import in wgpu** (geprüft):
  - wgpu-hal 30 hat `vulkan::Device::texture_from_dmabuf_fd(fd, desc, drm_modifier, stride, offset)`: unsafe, nur Unix, **nur Puffer mit einer Ebene**, Feature `VULKAN_EXTERNAL_MEMORY_DMA_BUF`.
  - Mit `create_texture_from_hal` wird daraus eine normale wgpu-Textur.
  - wgpu 30.0.0 erschien am 01.07.2026 (PR [#9366](https://github.com/gfx-rs/wgpu/pull/9366)). Mehrere Ebenen und GLES sind im offenen PR #10522.
  - [docs.rs](https://docs.rs/wgpu-hal/30.0.0/wgpu_hal/vulkan/struct.Device.html)
- **Explicit Sync:**
  - Auf NVIDIA Pflicht, seit Treiber 555 ([Phoronix](https://www.phoronix.com/news/Nvidia-555.58-Linux-Driver)).
  - Smithay `drm_syncobj`: Der `DrmSyncPointBlocker` hält den Commit zurück, bis der Acquire-Fence signalisiert ist. Das Protokoll nur anbieten, wenn `supports_syncobj_eventfd` true liefert ([docs](https://docs.rs/smithay/latest/smithay/wayland/drm_syncobj/index.html)).
  - wgpu kann keine fremden Semaphoren importieren; nicht nötig (R33).
- **Mipmaps:** Client-Puffer haben nur eine Mip-Stufe. Eine eigene Kopie mit Mip-Kette kostet etwa 33 % mehr Speicher und bei 1080p vermutlich unter 1 ms (unsicher).

### Eingaben und Pop-ups

- **Zeiger:**
  - UV-Koordinate in die Fenstergeometrie umrechnen (`xdg_surface.set_window_geometry`, ohne CSD-Schatten).
  - Die oberste (Sub-)Surface unter dem Punkt suchen.
  - `wl_pointer.enter/motion` mit surface-lokalen Koordinaten senden.
  - Den Zeiger zeichnet der Compositor selbst.
  - Für die Weltsteuerung gibt es `pointer_constraints` und `relative_pointer`.
- **Pop-ups (xdg_popup):**
  - Das Anker-Rechteck ist relativ zum Eltern-Surface.
  - Welche Fläche als Grenze gilt (Constraint-Fläche), legt der Compositor frei fest. Angepasst wird in der Reihenfolge Flip, Slide, Resize ([xdg-shell](https://wayland.app/protocols/xdg-shell)).
  - Bei einem Grab zählt ein Klick in die Welt als „außerhalb" und löst `popup_done` aus.
  - Tooltips in GTK4 sind Pop-ups ohne Grab.
- **Subsurfaces:** Am einfachsten zeichnet man den ganzen Surface-Baum eines Fensters in eine eigene Offscreen-Textur.
- **Dialoge:**
  - `xdg_toplevel.set_parent` bildet die Hierarchie.
  - `xdg-dialog-v1` ist nur ein Hinweis und noch im Status „testing"; Smithay hat es (R33).
- **Zwischenablage und Drag & Drop:** Smithay-Modul `selection`; Drag & Drop nutzt denselben Raycast.
- **IME:** `text-input-v3` und `input-method-v2`, beide experimentell.
- **HiDPI:** Mit `fractional-scale-v1` und `wp_viewporter` lässt sich die Texturauflösung unabhängig von der logischen Größe steuern.
- **Größenaushandlung:**
  - Die Größe im `configure` ist nur ein Hinweis; 0 heißt „Client wählt".
  - Bindend ist sie nur bei maximized und fullscreen bzw. während des Größenziehens (resizing).
  - Programme mit Mindestgröße liefern größere Puffer.
  - Tiled-States und serverseitige Dekoration (`xdg-decoration`) verhindern Titelleisten und Schatten an der Wand.

### XWayland und Wine

- **XWayland:**
  - Smithay hat `xwayland` und `X11Wm`. Override-Redirect-Fenster (Menüs) liegen in absoluten Root-Koordinaten und müssen dem Eltern-Fenster zugeordnet werden ([docs](https://docs.rs/smithay/latest/smithay/xwayland/xwm/trait.XwmHandler.html)).
  - Einfacher ist `xwayland-satellite` (niri startet es seit 25.08 automatisch): Es macht aus X11-Fenstern normale Wayland-Fenster. Programme mit festen Bildschirmkoordinaten können dabei brechen ([niri-Wiki](https://github.com/niri-wm/niri/wiki/Xwayland)).
- **Wine:**
  - Der Wayland-Treiber ist seit 9.22 in den Standard-Builds, gilt in 11.0 aber noch als experimentell. Ist `DISPLAY` gesetzt, wird zuerst X11 genommen (unsicher).
  - Parität mit X11 wird für Wine 12 erwartet.
  - Proton läuft standardmäßig über XWayland (`PROTON_ENABLE_WAYLAND=1` nur in Forks).

### Bildtaktung

- Zurückgehaltene Frame-Callbacks bremsen GTK, Qt, Browser und EGL mit Swap-Interval 1.
- mpv rendert ohne Callbacks gar nicht mehr.
- Spiele mit MAILBOX/IMMEDIATE oder Swap-Interval 0 bremsen nicht.
- `fifo-v1` und `commit-timing-v1` (Mesa ab 24.3): Der Compositor darf die fifo-Barriere für verdeckte Surfaces früher lösen ([fifo-v1](https://wayland.app/protocols/fifo-v1)).
- **Vorbilder für die Frame-Planung:**
  - niri: ein Frame pro Ausgabe gleichzeitig in Arbeit; ohne Schaden wird bis zum geschätzten VBlank gewartet ([Redraw Loop](https://niri-wm.github.io/niri/Development:-Redraw-Loop.html)).
  - KWin: misst die Renderzeit und startet möglichst spät vor dem VBlank.
- **Direct Scanout:** Der Vollbild-Puffer geht direkt auf die Primary Plane. Voraussetzung ist ein passender Modifier (dmabuf-Feedback).

### Alternativen und DRM-Video

- **Ohne eigenen Compositor** taugt das nicht:
  - PipeWire-ScreenCast braucht die Zustimmung des Nutzers und liefert keine Eingabe; RemoteDesktop arbeitet nur in Bildschirmkoordinaten.
  - X11 Composite hat keine Eingabe-Transformation.
- **DRM-Video:** Unter Linux gibt es nur Widevine L3 (Netflix maximal 720p). Ein Protokoll für geschützte Inhalte gibt es nicht; das Bild wird vermutlich nicht schwarz (unsicher).

## R10, R11: NVIDIA und Distribution

### NVIDIA-Treiber

- **Blackwell (RTX 50) läuft nur mit den offenen Kernelmodulen** ([NVIDIA-Forum](https://forums.developer.nvidia.com/t/rtx-50-series-blackwell-gpu-drivers-on-linux/335669)).
- **Treiberzweige:**
  - Production-Zweig R595, zuletzt 595.104.02 vom 01.10.2026. Seit 595 ist `modeset=1` Standard ([9to5linux](https://9to5linux.com/nvidia-595-linux-graphics-driver-released-as-latest-production-branch-version)).
  - Feature-Branch 610 (05/2026): mmap auf dmabufs, Modifier für mehrebenige Formate, Color-Pipeline. **Diese stört manche Compositors.**
  - Neueste Version: 615.78.08 (07.10.2026).
- **Unterstützung im Treiber:**
  - GBM seit R495.
  - dmabuf- und Modifier-Extensions seit R515.
  - Explicit Sync seit R555.
- **Eigenheiten beim Import:**
  - Der explizite Plane-Layout-Pfad lehnt Layouts ab, die der Listen-Pfad annimmt ([NVIDIA-Forum, 595.71.05](https://forums.developer.nvidia.com/t/371199)).
  - Getilte Modifier scheitern teils als Renderziel ([IonWL #3](https://codeberg.org/ideasman42/IonWL/issues/3)).
  - Cursor-Planes brauchen einen linearen Modifier.

### Compositors auf NVIDIA

- **Smithay:** offene NVIDIA-Issues:
  - #1988 VRAM-Erschöpfung
  - #1862 Flackern bei hoher Bildwiederholrate
  - #2208 nested winit antwortet einen Frame zu spät
- **niri:** hat bis heute kein `drm-syncobj` (#843). Dazu Flackern und VRAM-Lecks; empfohlen wird `GLVidHeapReuseRatio=0` ([Wiki](https://github.com/YaLTeR/niri/wiki/Nvidia)).
- **cosmic-comp:** Explicit Sync seit Alpha 6.1, aber 214 offene Issues zu NVIDIA.
- **Sway:** braucht weiterhin `--unsupported-gpu`.
- **NVK** (Mesa ab 25.2): unterstützt Blackwell, ist für den Alltag aber noch keine Alternative.

### Minecraft unter Linux

- Ab 26.3-snapshot-4 SDL3 statt GLFW; exklusiver Vollbildmodus stürzt unter Wayland ab ([minecraft.net](https://minecraft.net/en-us/article/minecraft-26-3-snapshot-4)).
- Ab 26.4-snapshot-1 ist Vulkan der Standard-Renderer, OpenGL bleibt als Rückfall (Sekundärquelle).
- Nativ unter Wayland gibt es auf NVIDIA Fehler; über XWayland läuft es robuster.

### Distributionen

| Platz | Distribution | Bewertung |
| --- | --- | --- |
| 1 | **Fedora KDE** (45 erscheint am 20.10.2026) | `akmod-nvidia` 595 mit offenem Modul über RPM Fusion ([Howto](https://rpmfusion.org/Howto/NVIDIA)); Secure Boot mit einmaliger MOK-Registrierung; Btrfs ab Werk, Snapper selbst einrichten; Erfahrungsbericht RTX 5070 stabil ([Blog](https://falcao.org/posts/nvidia-fedora-kde-wayland/)) |
| 2 | CachyOS | neueste Pakete, Snapshots ab Werk; Rolling Release, Secure Boot mühsam |
| 3 | Ubuntu 26.04 | Secure Boot am einfachsten; älteres Mesa, keine Snapshots |
| 4 | NixOS | Rollback ideal, Entwicklung mit viel Reibung |
| 5 | Tumbleweed | NVIDIA-Ausfall 04/2026 |
| 6 | Bazzite | schreibgeschütztes `/usr`, ungeeignet für Compositor-Entwicklung |

**Risiken bei Fedora:**
- Ein Kernel-Update kann vor dem fertigen akmod-Build landen; dann bleibt der Bildschirm schwarz. Gegenmittel: Snapshots, versionlock.
- Aufwachen aus dem Ruhezustand hängt mit Kernel 7.x auf Blackwell; Umgehung `s2idle`.

## R13, R16: Sprache, Grafik-API, Entwicklungsumgebung

### Vergleich der Stacks

| Stack | Bewertung |
| --- | --- |
| **Rust + wgpu** | passt am besten: Smithay und die Formate-Crates sind Rust, dmabuf-Import offiziell seit wgpu 30, wgpu-hal öffnet den Weg zu rohem Vulkan |
| Rust + ash/vulkano | erst wählen, wenn wgpu nachweislich blockiert; ash 0.38 ist von 04/2024, vulkano 0.35.2 von 08/2025 |
| C++ + Vulkan | wenige Bibliotheken für Minecraft-Formate, hohe Lernkurve |
| Godot | braucht einen Engine-Fork (wie SimulaVR) |
| Bevy | fremde Schleife, Breaking Changes mit jeder Version (0.20 vom 10/2026) |

### Crates für Minecraft-Formate

Abfrage vom 16.10.2026:

| Crate | Stand | Bewertung |
| --- | --- | --- |
| fastnbt | 2.6.3 | aktiv |
| simdnbt | 0.10.0 | aktiv, sehr schnell |
| azalea | 0.16.0+mc26.1, unterstützt MC 26.2 | aktiv, aber Grafik ist kein Ziel; `azalea-registry` und `azalea-block` liefern Blockdaten |
| fastanvil | 0.32.0 vom 08/2025, vor 1.0 | ob es 26.x liest, ist unsicher |
| valence | letztes Release 2023 | praktisch tot |

**Vorbilder** (Code nicht übernehmen):
- wgpu-mc (MPL-2.0)
- Pomme (Rust-Client mit Vulkan und Smooth Lighting, GPL-3, [Repo](https://github.com/PommeMC/Client))

### Shader-Packs und Entwicklungsumgebung

- **Shader-Packs:**
  - Packs sind oft in GLSL 120 oder im Compatibility-Profil geschrieben, neuere in GLSL 460.
  - Jeder Stack braucht einen eigenen Umschreibschritt auf GLSL 450 mit Vulkan-Semantik, danach shaderc. shaderc ist robuster als der GLSL-Eingang von naga, der nur GLSL 440+ kann.
  - Iris nutzt glsl-transformer (AGPL-3).
  - Mojangs Umstieg auf Vulkan kann das Format der Packs ändern.
- **Entwicklungsumgebung:**
  - Der Weltrenderer liefe mit wgpu auch unter Windows. Das ist kein Ziel mehr, T5 ist entschieden: alles unter Linux.
  - WSL2 taugt nicht für den Compositor: kein DRM, Vulkan nur über dzn.

## R32, R33: Waylandcraft, Sync und Import im Detail (18.10.2026)

Am Quellcode geprüft (Git-Stände vom 15.–17.10.2026).

### Waylandcraft (R32)

- Rust-Teil (~5.600 Zeilen) als JNI-Bibliothek, Smithay auf Commit `89e58f7` mit `default-features = false`, Features `wayland_frontend` und `backend_drm`.
- **Kein `drm_syncobj`**; trotzdem auf NVIDIA lauffähig, nur mit `__GL_THREADED_OPTIMIZATIONS=0` für Minecraft (das Programm entfernt die Variable für gestartete Apps).
- dmabuf-Import per `eglCreateImage` (`EGL_LINUX_DMA_BUF_EXT`) und `glEGLImageTargetTexture2DOES`; dmabuf-Feedback mit den Formaten aus Minecrafts GL-Kontext.
- `WorldPlane`: Ursprung plus drei Achsen; Schnitt Strahl–Ebene, dann inverse 3×3-Matrix → lokale Koordinate. Parallelstrahl (Nenner 0) und Treffer hinter der Kamera → kein Treffer.
- Pixel pro Block: Standard **500**.
- Pop-ups: eigene Anzeige pro Pop-up; Positioner-Geometrie unverändert übernommen, Grab-Anfragen ignoriert.
- Tastatur: `G` = Fang, `Alt+Q` = harter Fang (auch Escape und relative Maus), erneut `Alt+Q` gibt frei.
- [Repo](https://github.com/EVV1E/waylandcraft)

### Explicit Sync in Smithay (R33)

- `drm_syncobj`: Acquire wird per eventfd abgewartet (`DrmSyncPointBlocker`), der Commit gilt erst danach. Release wird signalisiert, wenn alle Referenzen auf `backend::renderer::utils::Buffer` freigegeben sind, oder wenn ein neuer Commit den alten Punkt ersetzt.
- Braucht ein `DrmDeviceFd` (Feature `backend_drm`).
- [Quelle `src/wayland/drm_syncobj/mod.rs`](https://github.com/Smithay/smithay/blob/master/src/wayland/drm_syncobj/mod.rs)

### dmabuf-Import in wgpu (R33)

- Aktuell: wgpu 30.0.1 (21.08.2026). `texture_from_dmabuf_fd` nutzt `VkImageDrmFormatModifierExplicitCreateInfoEXT` (expliziter Pfad, eine Ebene, Offset und Stride); Vulkan übernimmt den fd.
- Im Trunk nach 30.0.1: `hal::TextureBarrier::queue_family_ownership_transfer` mit `QueueFamily::External`/`Foreign` ([PR #9668](https://github.com/gfx-rs/wgpu/pull/9668), 14.08.2026) und `Texture::mark_externally_initialized` (PR #10075).
- Offener [PR #10522](https://github.com/gfx-rs/wgpu/pull/10522): GLES-Import, Formatabfrage, Acquire/Release-Helfer; Autor baut einen Smithay-Compositor mit wgpu. Maintainer wollen die Formatabfrage eher nicht in wgpu.
- Kein Import fremder Semaphoren in wgpu-hal.
- NVIDIA-Fehler im expliziten Pfad: gemeldet für **NV12** (mehrere Ebenen) auf 595.71.05, Listen-Pfad und EGL gehen; Nutzerbericht, keine Antwort von NVIDIA ([Forum](https://forums.developer.nvidia.com/t/nvidia-vulkan-vkimagedrmformatmodifierexplicitcreateinfoext-rejects-nv12-layouts-that-the-list-based-path-accepts/371199)).
- Smithays winit-Backend verlangt `R: From<GlesRenderer>`; für wgpu ungeeignet.

### Sonstiges (R33)

- `xwayland-satellite` (MPL-2.0, 0.8.2 in Fedora 44) bildet Override-Redirect-Fenster per Heuristik als `xdg_popup` ab ([Repo](https://github.com/Supreeeme/xwayland-satellite)).
- Smithay hat `xdg-dialog-v1` (`wayland::shell::xdg::dialog`).
- Wie wgpu-Bilder auf Smithays DRM-Ausgabe kommen: nicht recherchiert, bleibt Prototyp-Test P15 (m0).

### NVIDIA 610+ und Fedora 44

- 610.43.02 bringt die DRM-Color-Pipeline pro Plane; Compositors, die nicht umgehbare Colorops falsch behandeln, zeigen beim Einschalten von HDR einen schwarzen Bildschirm. Abhilfe: neuer Parameter `color_pipeline` von `nvidia-drm` ([Release-Notes](https://download.nvidia.com/XFree86/Linux-x86_64/610.43.02/README/wayland-issues.html), [GamingOnLinux](https://www.gamingonlinux.com/2026/05/nvidia-driver-610-43-02-arrives-for-linux-with-vulkan-upgrades-drm-colour-pipeline-api-support/page=1/)).
- RPM Fusion für Fedora 44 hat keinen eigenen 595-Zweig zum Festhalten, nur Legacy 580xx ([Fedora Discussion](https://discussion.fedoraproject.org/t/pascal-nvidia-gpus-no-longer-supported-as-of-595-xx-version-from-rpmfusion/190474)).

## R26: Recht

Kein Rechtsrat; zusammengefasst ist nur, was die Dokumente sagen.

**Quellen:** [EULA](https://www.minecraft.net/en-us/eula), [Usage Guidelines](https://www.minecraft.net/en-us/usage-guidelines).

- **Werkzeuge und Assets:**
  - Die EULA erlaubt Werkzeuge, solange sie nicht offiziell wirken.
  - Die Weitergabe von Spielsoftware und -inhalten ist verboten.
  - Assets zur Laufzeit lokal zu lesen, regeln die Dokumente nicht ausdrücklich.
- **Wie es andere machen:**
  - Chunky nimmt das neueste Jar aus `.minecraft/versions`.
  - BlueMap lädt das Client-Jar erst nach Zustimmung von Mojangs Servern.
  - Fallback-Texturen müssen eigene sein.
- **Datengenerator:**
  - Er steckt im server.jar: `java -DbundlerMainClass="net.minecraft.data.Main" -jar server.jar --reports` ([Wiki](https://minecraft.wiki/w/Tutorial:Running_the_data_generator)).
  - EULA und Richtlinien erwähnen ihn nicht.
  - Den server.jar beim Nutzer zu installieren ist erlaubt, ihn mitzuliefern nicht.
- **Name:**
  - „Minecraft" darf nicht der Haupt- oder prägende Name sein, nur ein beschreibender Zusatz.
  - Keine Logos.
  - Bei einer Veröffentlichung ist der Hinweis „NOT AN OFFICIAL MINECRAFT [PRODUCT]. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT" Pflicht.
