# M7: Mod- und Shader-Support

**Planungsstand:** grob, nicht bereit. Es fehlen:
- R29: Mod-Formate, Modloader, Auswahl der Shader und deren Lizenzen.

**Größe:** nicht abschätzbar, Risiko sehr hoch. Wahrscheinlich eine der letzten Stufen.
**Voraussetzung:** M2. Die Render-Pipeline (T8) muss erweiterbar sein.
**Features:** [features.md](../features.md) §3 „Mods, Shader, modifizierte Welten" (Ziel, Umfang des Adapters, Machbarkeit). **Belege:** recherche-ergebnisse.md R13.

## Kern: darauf kommt es an

1. **Shader-Übersetzung** heißt im Kern Nachbauen. Shader-Packs sind mehrstufige OpenGL-Pipelines mit G-Buffern; die eigene Pipeline muss solche Durchgänge abbilden können.
2. **Lizenzen:** Jeder Shader-Pack hat seine eigene Lizenz. Übersetzter Code fällt unter die Lizenz des Originals, und das Projekt steht unter 0BSD. Deshalb nur Packs mit verträglicher Lizenz oder ausdrücklicher Erlaubnis verwenden.

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
