# M7: Mod- und Shader-Support

**Planungsstand:** grob, nicht bereit. Es fehlen:
- R29: Mod-Formate, Modloader, Auswahl der Shader und deren Lizenzen.

**Größe laut Featureliste:** nicht abschätzbar, Risiko sehr hoch. Wahrscheinlich eine der letzten Stufen.
**Voraussetzung:** M2. Die Render-Pipeline (T8) muss erweiterbar sein.

> **Für die Implementierungs-KI:** Diese Datei enthält alles, was für M7 nötig ist. Globale Leitplanken stehen in [CLAUDE.md](../CLAUDE.md), Quellen in [recherche-ergebnisse.md](recherche-ergebnisse.md), Abschnitt R13.

## Ziel

- **Mods:** Ein Adapter liest aus Mods das Wichtige: Custom-Blöcke, Texturen, eventuell Custom-Mobs. Rezepte, Blockverhalten und Texturanimationen der Mods fallen weg.
- **Shader:** Es gibt einen eingebauten Shader, vielleicht in mehreren Versionen. Dafür werden bekannte Minecraft-Shader-Packs in eigenen Shadercode übersetzt und verschlankt. Hat der Nutzer einen dieser Packs für sein echtes Minecraft installiert, nutzt die Shell ihn ebenfalls.

## Kern: darauf kommt es an

1. **Shader-Übersetzung** heißt im Kern Nachbauen. Shader-Packs sind mehrstufige OpenGL-Pipelines mit G-Buffern; die eigene Pipeline muss solche Durchgänge abbilden können.
2. **Lizenzen:** Jeder Shader-Pack hat seine eigene Lizenz. Übersetzter Code fällt unter die Lizenz des Originals, und das Projekt steht unter 0BSD. Deshalb nur Packs mit verträglicher Lizenz oder ausdrücklicher Erlaubnis verwenden.
3. **Mods:** Blöcke und Texturen aus den Asset-Dateien der Mods zu lesen ist machbar. Blöcke mit eigenem Darstellungscode und Mods, die das Rendering verändern, sind kaum machbar.

## Entschieden

- Eingebauter, eigener Shader. Übersetzte und verschlankte Versionen bekannter Shader bilden eine Liste unterstützter Shader, ähnlich wie bei den Mods.
- Direkte Kompatibilität mit beliebigen Shader-Packs (OptiFine/Iris) ist **kein** Ziel.
- Mod-Adapter: nur Custom-Blöcke, Texturen und eventuell Custom-Mobs.

## Technische Grundlagen

Aus R13, mit R29 vertiefen.

- **Sprachstand der Packs:** Packs sind oft in GLSL 120 oder im Compatibility-Profil geschrieben, neuere in GLSL 460 mit Compute und SSBOs.
- **Übersetzungsweg:** eigener Umschreibschritt auf GLSL 450 mit Vulkan-Semantik, dann shaderc nach SPIR-V, dann in wgpu als Passthrough. Der GLSL-Eingang von naga kann nur GLSL 440+ und ist dafür riskant.
- **Iris** schreibt Packs mit glsl-transformer (AGPL-3) um. Das Werkzeug ist mit 0BSD nicht verträglich und darf nicht eingebunden werden.
- **Mojangs Umstieg auf Vulkan** (Standard ab den 26.4-Snapshots) kann das Format der Shader-Packs ändern.
- **Mods:** Fabric- und NeoForge-Jars enthalten Assets im selben Aufbau wie Vanilla (`assets/<modid>/…`). Prüfen in R29.

## Unbedingt beachten

- Keine Pack-Dateien mitliefern, nur eigene Übersetzungen mit geklärter Lizenz.
- Unbekannte Mod-Blöcke werden zu Platzhaltern (wie in M1), nie zum Absturz.

## Abnahme

Konkretisieren, wenn der Meilenstein geplant wird.
