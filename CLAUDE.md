# CLAUDE.md — StoryLifeUtils

Application desktop Tauri 2 pour automatiser le mode AFK sur le serveur StoryLife (FiveM).
Frontend React 19 + TypeScript + Tailwind CSS 4, backend Rust avec API Windows native.

## Commandes
- Dev frontend : `npm run dev` (Vite, port 1420)
- Dev complet : `npm run tauri dev`
- Build : `npm run tauri build` (NSIS installer Windows)
- TypeScript check : `tsc --noEmit`

## Architecture

### Frontend (`src/`)
- Composants React : `src/components/`
- Hooks custom : `src/hooks/`
- Utilitaires : `src/lib/`
- Styles : `src/styles/globals.css` — thème dark custom avec variables CSS
- Point d'entrée : `src/main.tsx`
- Tailwind CSS 4 via plugin Vite (`@tailwindcss/vite`)
- Icônes : `lucide-react`

### Backend Tauri (`src-tauri/`)
- State & types : `src-tauri/src/state.rs` — `AutomationState`, `AutomationStatus`, `Config`, `WindowInfo`
- Commandes Tauri : `src-tauri/src/commands/` — un module par domaine (`config.rs`, `automation.rs`)
- Modules d'automatisation : `src-tauri/src/automation/`
  - `window_finder.rs` — détection fenêtre FiveM via Win32 `EnumWindows`
  - `process_manager.rs` — kill/launch des process FiveM
  - `screen_capture.rs` — capture d'écran via GDI (BitBlt)
  - `ocr.rs` — détection texte via Windows OCR (`Media.Ocr`)
  - `character_selector.rs` — sélection automatique du personnage
  - `input_sender.rs` — envoi de touches anti-AFK
- Config persistée dans `%APPDATA%/StoryLifeUtils/config.json`

### Machine à états
Le cœur du backend suit un cycle : `Idle → SearchingWindow → LaunchingFivem → WaitingOcr → SelectingCharacter → SendingKeys → AfkActive → Reconnecting → ...`
État partagé via `Arc<Mutex<AutomationStatus>>` + signal d'arrêt `Arc<Mutex<bool>>`.

## Code — Frontend
- React 19 avec hooks (`useState`, `useEffect`, custom hooks dans `src/hooks/`)
- Communication avec le backend uniquement via `@tauri-apps/api` (`invoke`)
- Ne jamais appeler de commande Tauri directement dans le JSX — passer par un hook ou un handler
- Tailwind CSS pour le styling, pas de CSS-in-JS
- Thème dark obligatoire (palette définie dans `globals.css`)

## Code — Backend (Rust)
- Toutes les interactions Windows passent par la crate `windows` (0.59), pas de FFI brut
- Les commandes Tauri (`#[tauri::command]`) sont dans `src-tauri/src/commands/`
- Les modules d'automatisation dans `src-tauri/src/automation/` ne dépendent jamais de Tauri directement
- Gestion d'erreurs : retourner `Result<T, String>` pour les commandes Tauri
- Logging via `log` + `env_logger`
- Concurrence : `tokio` pour l'async, `Arc<Mutex<>>` pour l'état partagé

## Workflow
- Plan Mode pour toute tâche 3+ étapes
- Si ça déraille : STOP, re-planifier avant de continuer
- Toujours vérifier `tsc --noEmit` après modification frontend

## Erreurs à ne pas reproduire
- Pas de `unsafe` sans commentaire expliquant pourquoi c'est safe
- Ne pas oublier de libérer les handles Windows (DC, bitmap, process handles) — toujours cleanup dans le bon ordre
- Les appels Win32 doivent vérifier le retour avant de continuer (pas de `.unwrap()` sur du Win32)
- Ne pas hardcoder de HWND ou PID — toujours les résoudre dynamiquement
- Backend : ne pas bloquer le thread principal Tauri — utiliser `tokio::spawn` pour les tâches longues
- Frontend : toujours gérer les 3 états (loading / error / success) sur les appels `invoke`
