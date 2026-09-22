<script lang="ts">
  import { tick } from 'svelte';
  import { themes, currentTheme, setTheme } from '$lib/stores/theme';
  import { soundState, toggleSound, setVolume, playSound } from '$lib/audio';
  import { appConfig, saveConfig, type AppConfig } from '$lib/stores/config';
  import { refreshPing, refreshGeo, setGeoEnabled, isGeoEnabled } from '$lib/stores/network';

  export let show = false;
  export let activeTab: 'commands' | 'settings' = 'commands';

  export let toggleFileManager: () => void;
  export let toggleProcesses: () => void;
  export let toggleKeyboard: () => void;
  export let toggleGlobe: () => void;
  export let toggleFullscreen: () => void;
  export let addTab: () => void;
  export let closeTab: () => void;

  let query = '';
  let selectedIndex = 0;
  let searchInput: HTMLInputElement;
  let saveStatus = '';

  let configCopy: AppConfig = { ...$appConfig };
  $: if (show) {
    configCopy = { ...$appConfig, audioEnabled: $soundState.enabled, audioVolume: $soundState.volume };
  }

  interface CommandItem {
    id: string;
    title: string;
    category: string;
    shortcut?: string;
    action: () => void;
  }

  $: commands = [
    // Panels
    { id: 'toggle-fm', title: 'Toggle File Browser', category: 'PANELS', shortcut: 'Ctrl+Shift+B', action: toggleFileManager },
    { id: 'toggle-procs', title: 'Toggle Process Monitor', category: 'PANELS', shortcut: 'Ctrl+Shift+P', action: toggleProcesses },
    { id: 'toggle-osk', title: 'Toggle On-Screen Keyboard', category: 'PANELS', shortcut: 'Ctrl+Shift+K', action: toggleKeyboard },
    { id: 'toggle-globe', title: 'Toggle World View Globe', category: 'PANELS', shortcut: 'Ctrl+Shift+G', action: toggleGlobe },
    { id: 'toggle-fs', title: 'Toggle Fullscreen Mode', category: 'PANELS', shortcut: 'Ctrl+Shift+F', action: toggleFullscreen },

    // Terminal
    { id: 'new-tab', title: 'New Terminal Tab', category: 'TERMINAL', shortcut: 'Ctrl+Shift+T', action: addTab },
    { id: 'close-tab', title: 'Close Current Tab', category: 'TERMINAL', shortcut: 'Ctrl+Shift+W', action: closeTab },
    { id: 'font-inc', title: 'Increase Font Size (+1px)', category: 'TERMINAL', action: () => updateFontSize(1) },
    { id: 'font-dec', title: 'Decrease Font Size (-1px)', category: 'TERMINAL', action: () => updateFontSize(-1) },
    { id: 'font-reset', title: 'Reset Font Size (14px)', category: 'TERMINAL', action: () => setFontSize(14) },

    // Themes
    { id: 'theme-default', title: 'Switch Theme: Default (Cyan / Dark Blue)', category: 'THEMES', action: () => selectTheme('default') },
    { id: 'theme-tron', title: 'Switch Theme: Tron (Neon Orange)', category: 'THEMES', action: () => selectTheme('tron') },
    { id: 'theme-blade', title: 'Switch Theme: Blade (Amber / Gold)', category: 'THEMES', action: () => selectTheme('blade') },
    { id: 'theme-matrix', title: 'Switch Theme: Matrix (Phosphor Green)', category: 'THEMES', action: () => selectTheme('matrix') },

    // Audio
    { id: 'audio-toggle', title: 'Toggle Audio SFX (Mute/Unmute)', category: 'AUDIO', shortcut: 'Ctrl+Shift+S', action: () => toggleSound() },
    { id: 'audio-vol-25', title: 'Set Volume to 25%', category: 'AUDIO', action: () => setVolume(0.25) },
    { id: 'audio-vol-50', title: 'Set Volume to 50%', category: 'AUDIO', action: () => setVolume(0.50) },
    { id: 'audio-vol-75', title: 'Set Volume to 75%', category: 'AUDIO', action: () => setVolume(0.75) },
    { id: 'audio-vol-100', title: 'Set Volume to 100%', category: 'AUDIO', action: () => setVolume(1.0) },

    // Diagnostics & Net
    { id: 'net-ping', title: 'Ping Test (Probe 1.1.1.1 Latency)', category: 'NETWORK', action: () => { refreshPing(); playSound('granted'); } },
    { id: 'net-geo', title: 'Refresh Geolocation Coordinates', category: 'NETWORK', action: () => {
      if (!isGeoEnabled()) { playSound('denied'); return; }
      refreshGeo(); playSound('granted');
    } },

    // Settings
    { id: 'open-settings', title: 'Open Settings & Preferences GUI', category: 'CONFIG', action: () => { activeTab = 'settings'; } },
  ] as CommandItem[];

  $: filteredCommands = query.trim()
    ? commands.filter((c) =>
        c.title.toLowerCase().includes(query.toLowerCase()) ||
        c.category.toLowerCase().includes(query.toLowerCase()) ||
        (c.shortcut && c.shortcut.toLowerCase().includes(query.toLowerCase()))
      )
    : commands;

  $: if (filteredCommands.length > 0 && selectedIndex >= filteredCommands.length) {
    selectedIndex = 0;
  }

  function close() {
    show = false;
    query = '';
    selectedIndex = 0;
    saveStatus = '';
    playSound('denied');
  }

  function execute(cmd: CommandItem) {
    playSound('granted');
    cmd.action();
    if (cmd.id !== 'open-settings') {
      close();
    }
  }

  function selectTheme(key: string) {
    setTheme(key);
    configCopy.theme = key;
    appConfig.update((c) => ({ ...c, theme: key }));
    playSound('theme');
  }

  function updateFontSize(delta: number) {
    appConfig.update((c) => {
      const next = Math.max(10, Math.min(24, c.fontSize + delta));
      configCopy.fontSize = next;
      return { ...c, fontSize: next };
    });
  }

  function setFontSize(size: number) {
    appConfig.update((c) => {
      configCopy.fontSize = size;
      return { ...c, fontSize: size };
    });
  }

  function toggleGeo() {
    const enabled = !$appConfig.geoLookup;
    configCopy.geoLookup = enabled;
    setGeoEnabled(enabled);
    // Persisted straight away rather than on Save: turning a privacy setting
    // off must not quietly revert on the next launch.
    saveConfig({ ...$appConfig, geoLookup: enabled });
    playSound(enabled ? 'granted' : 'denied');
  }

  async function handleSaveConfig() {
    configCopy.audioEnabled = $soundState.enabled;
    configCopy.audioVolume = $soundState.volume;
    const ok = await saveConfig(configCopy);
    if (ok) {
      playSound('granted');
      saveStatus = 'SAVED TO DISK // ~/.config/nextgen-ui/config.toml';
      setTimeout(() => { saveStatus = ''; }, 3000);
    } else {
      playSound('error');
      saveStatus = 'ERROR SAVING CONFIG';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!show) return;

    if (e.key === 'Escape') {
      e.preventDefault();
      close();
      return;
    }

    if (activeTab === 'commands') {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectedIndex = (selectedIndex + 1) % Math.max(1, filteredCommands.length);
        playSound('stdin', { volumeScale: 0.2 });
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectedIndex = (selectedIndex - 1 + filteredCommands.length) % Math.max(1, filteredCommands.length);
        playSound('stdin', { volumeScale: 0.2 });
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          execute(filteredCommands[selectedIndex]);
        }
      }
    }
  }

  $: if (show && searchInput) {
    tick().then(() => searchInput?.focus());
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div class="palette-backdrop" role="presentation" on:click|self={close}>
    <div class="palette-modal">
      <div class="modal-corners tl"></div><div class="modal-corners tr"></div>
      <div class="modal-corners bl"></div><div class="modal-corners br"></div>

      <!-- Header tabs -->
      <div class="palette-header">
        <div class="tab-triggers">
          <button
            type="button"
            class="tab-btn"
            class:active={activeTab === 'commands'}
            on:click={() => { activeTab = 'commands'; playSound('folder'); }}
          >
            &gt;_ COMMAND PALETTE
          </button>
          <button
            type="button"
            class="tab-btn"
            class:active={activeTab === 'settings'}
            on:click={() => { activeTab = 'settings'; playSound('folder'); }}
          >
            ⚙ PREFERENCES &amp; CONFIG
          </button>
        </div>
        <button type="button" class="close-btn" on:click={close} title="Close (Esc)">✕</button>
      </div>

      {#if activeTab === 'commands'}
        <!-- Search Input -->
        <div class="search-bar">
          <span class="prompt-arrow">&gt;</span>
          <input
            bind:this={searchInput}
            type="text"
            bind:value={query}
            placeholder="Type a command, theme, panel, or shortcut..."
            spellcheck="false"
            autocomplete="off"
          />
          {#if query}
            <button type="button" class="clear-query" on:click={() => { query = ''; searchInput?.focus(); }}>✕</button>
          {/if}
        </div>

        <!-- Command List -->
        <div class="command-list">
          {#if filteredCommands.length === 0}
            <div class="empty-results">// NO COMMANDS MATCHING &quot;{query}&quot;</div>
          {:else}
            {#each filteredCommands as cmd, i}
              <button
                type="button"
                class="command-row"
                class:selected={i === selectedIndex}
                on:mouseenter={() => { selectedIndex = i; }}
                on:click={() => execute(cmd)}
              >
                <span class="cmd-category">[{cmd.category}]</span>
                <span class="cmd-title">{cmd.title}</span>
                {#if cmd.shortcut}
                  <span class="cmd-shortcut">{cmd.shortcut}</span>
                {/if}
              </button>
            {/each}
          {/if}
        </div>
        <div class="palette-footer">
          <span>↑↓ NAVIGATE</span>
          <span>↵ EXECUTE</span>
          <span>ESC CLOSE</span>
        </div>
      {:else}
        <!-- Settings Panel -->
        <div class="settings-body">
          <!-- Theme section -->
          <div class="settings-group">
            <div class="group-title">◈ ACTIVE THEME</div>
            <div class="theme-grid">
              {#each Object.entries(themes) as [key, t]}
                <button
                  type="button"
                  class="theme-card"
                  class:active={$currentTheme.name === t.name}
                  style="--theme-accent: {t.accent}"
                  on:click={() => selectTheme(key)}
                >
                  <span class="swatch" style="background: {t.accent}; box-shadow: 0 0 8px {t.accent}"></span>
                  <span class="theme-title">{t.name}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Typography / Font -->
          <div class="settings-group">
            <div class="group-title">◱ TERMINAL DISPLAY</div>
            <div class="form-row">
              <label for="font-size-slider">Font Size ({$appConfig.fontSize}px):</label>
              <input
                id="font-size-slider"
                type="range"
                min="10"
                max="22"
                step="1"
                bind:value={configCopy.fontSize}
                on:input={() => setFontSize(configCopy.fontSize)}
              />
              <span class="val-tag">{$appConfig.fontSize}px</span>
            </div>
            <div class="form-row">
              <label for="stats-width-slider">Telemetry Panel Width ({$appConfig.statsWidth}px):</label>
              <input
                id="stats-width-slider"
                type="range"
                min="180"
                max="320"
                step="10"
                bind:value={configCopy.statsWidth}
                on:input={() => appConfig.update((c) => ({ ...c, statsWidth: configCopy.statsWidth }))}
              />
              <span class="val-tag">{$appConfig.statsWidth}px</span>
            </div>
          </div>

          <!-- Audio Settings -->
          <div class="settings-group">
            <div class="group-title">🔊 SOUND DESIGN (eDEX-UI AUTHENTIC)</div>
            <div class="form-row">
              <label for="audio-toggle-btn">Audio Sound Effects:</label>
              <button
                id="audio-toggle-btn"
                type="button"
                class="toggle-switch"
                class:on={$soundState.enabled}
                on:click={() => toggleSound()}
              >
                {$soundState.enabled ? 'ENABLED' : 'MUTED'}
              </button>
            </div>
            <div class="form-row">
              <label for="audio-vol-slider">Master Volume ({Math.round($soundState.volume * 100)}%):</label>
              <input
                id="audio-vol-slider"
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={$soundState.volume}
                on:input={(e) => setVolume(parseFloat(e.currentTarget.value))}
              />
              <span class="val-tag">{Math.round($soundState.volume * 100)}%</span>
            </div>
          </div>

          <!-- Privacy -->
          <div class="settings-group">
            <div class="group-title">⛨ PRIVACY</div>
            <div class="form-row">
              <label for="geo-toggle-btn">Public IP &amp; Geolocation Lookup:</label>
              <button
                id="geo-toggle-btn"
                type="button"
                class="toggle-switch"
                class:on={$appConfig.geoLookup}
                on:click={toggleGeo}
              >
                {$appConfig.geoLookup ? 'ENABLED' : 'DISABLED'}
              </button>
            </div>
            <div class="privacy-note">
              When enabled, your IP is sent to ipwho.is to place you on the globe,
              and your public IP and city are shown on screen.
            </div>
          </div>

          <!-- Shell path -->
          <div class="settings-group">
            <div class="group-title">⌨ SHELL PROCESS</div>
            <div class="form-row">
              <label for="shell-input">Shell Executable Path:</label>
              <input
                id="shell-input"
                type="text"
                class="text-input"
                bind:value={configCopy.shell}
                placeholder="/bin/bash (empty for default)"
              />
            </div>
          </div>

          <!-- Shortcuts Cheat Sheet -->
          <div class="settings-group">
            <div class="group-title">⌨ GLOBAL HUD SHORTCUTS</div>
            <div class="shortcuts-grid">
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+C</span><span class="sc-desc">Command Palette &amp; Settings</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+T</span><span class="sc-desc">New Terminal Tab</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+W</span><span class="sc-desc">Close Active Tab</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+B</span><span class="sc-desc">Toggle File Manager</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+P</span><span class="sc-desc">Toggle Processes</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+K</span><span class="sc-desc">Toggle On-Screen Keyboard</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+G</span><span class="sc-desc">Toggle World Globe</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+S</span><span class="sc-desc">Toggle Audio FX</span></div>
              <div class="sc-item"><span class="sc-key">Ctrl+Shift+F</span><span class="sc-desc">Toggle Fullscreen</span></div>
            </div>
          </div>

          <!-- Action bar -->
          <div class="save-bar">
            {#if saveStatus}
              <span class="save-status">{saveStatus}</span>
            {/if}
            <button type="button" class="save-btn" on:click={handleSaveConfig}>
              [ SAVE CONFIG TO DISK // CONFIG.TOML ]
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99999;
    background: rgba(4, 7, 14, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 60px;
  }

  .palette-modal {
    position: relative;
    width: 680px;
    max-height: 82vh;
    display: flex;
    flex-direction: column;
    background: var(--panel-bg, #0a0e1af2);
    border: 1px solid var(--panel-border, #00c8ff44);
    border-radius: 6px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8), 0 0 30px var(--accent-glow, #00c8ff22);
    overflow: hidden;
  }

  .modal-corners {
    position: absolute;
    width: 12px;
    height: 12px;
    border-color: var(--accent, #00c8ff);
    border-style: solid;
    pointer-events: none;
    z-index: 2;
  }
  .tl { top: 2px; left: 2px; border-width: 2px 0 0 2px; }
  .tr { top: 2px; right: 2px; border-width: 2px 2px 0 0; }
  .bl { bottom: 2px; left: 2px; border-width: 0 0 2px 2px; }
  .br { bottom: 2px; right: 2px; border-width: 0 2px 2px 0; }

  .palette-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--panel-border, #00c8ff22);
    padding: 6px 12px;
    background: rgba(0, 0, 0, 0.3);
  }

  .tab-triggers {
    display: flex;
    gap: 8px;
  }

  .tab-btn {
    background: none;
    border: none;
    font-family: inherit;
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--fg-dim, #c8d8f055);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .tab-btn:hover { color: var(--fg, #c8d8f0); }
  .tab-btn.active {
    color: var(--accent, #00c8ff);
    background: var(--accent-glow, #00c8ff18);
    border: 1px solid var(--accent, #00c8ff44);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--fg-dim, #c8d8f066);
    font-size: 13px;
    cursor: pointer;
    padding: 4px 8px;
  }
  .close-btn:hover { color: var(--red, #ff4c6a); }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--panel-border, #00c8ff22);
    background: rgba(0, 0, 0, 0.2);
  }

  .prompt-arrow {
    color: var(--accent, #00c8ff);
    font-size: 14px;
    font-weight: bold;
  }

  .search-bar input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-family: inherit;
    font-size: 12px;
    color: var(--fg, #c8d8f0);
    letter-spacing: 0.08em;
  }
  .search-bar input::placeholder {
    color: var(--fg-dim, #c8d8f033);
  }

  .clear-query {
    background: none;
    border: none;
    color: var(--fg-dim, #c8d8f044);
    cursor: pointer;
    font-size: 11px;
  }

  .command-list {
    flex: 1;
    overflow-y: auto;
    max-height: 420px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .command-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.1s;
    border: 1px solid transparent;
    background: none;
    font-family: inherit;
    text-align: left;
    width: 100%;
  }
  .command-row:hover, .command-row.selected {
    background: var(--accent-glow, #00c8ff18);
    border-color: var(--accent, #00c8ff33);
  }

  .cmd-category {
    font-size: 8px;
    letter-spacing: 0.12em;
    color: var(--accent-dim, #00c8ff88);
    width: 80px;
  }

  .cmd-title {
    flex: 1;
    font-size: 11px;
    letter-spacing: 0.06em;
    color: var(--fg, #c8d8f0);
  }

  .cmd-shortcut {
    font-size: 9px;
    color: var(--fg-dim, #c8d8f055);
    background: rgba(255, 255, 255, 0.06);
    padding: 2px 6px;
    border-radius: 3px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .empty-results {
    padding: 24px;
    text-align: center;
    font-size: 10px;
    color: var(--fg-dim, #c8d8f044);
    letter-spacing: 0.1em;
  }

  .palette-footer {
    display: flex;
    gap: 16px;
    padding: 6px 14px;
    border-top: 1px solid var(--panel-border, #00c8ff18);
    background: rgba(0, 0, 0, 0.3);
    font-size: 8px;
    letter-spacing: 0.1em;
    color: var(--fg-dim, #c8d8f033);
  }

  /* Settings GUI */
  .settings-body {
    flex: 1;
    overflow-y: auto;
    max-height: 520px;
    padding: 14px 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .privacy-note { font-size: 9px; color: var(--fg-dim, #c8d8f066); line-height: 1.4; }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--panel-border, #00c8ff18);
  }

  .group-title {
    font-size: 10px;
    letter-spacing: 0.14em;
    color: var(--accent, #00c8ff);
    font-weight: bold;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  .theme-card {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--panel-border, #00c8ff22);
    cursor: pointer;
    font-family: inherit;
    transition: all 0.15s;
  }
  .theme-card:hover { border-color: var(--theme-accent); }
  .theme-card.active {
    border-color: var(--theme-accent);
    background: rgba(255, 255, 255, 0.05);
    box-shadow: 0 0 10px var(--accent-glow, #00c8ff22);
  }

  .swatch {
    width: 12px;
    height: 12px;
    border-radius: 2px;
    display: inline-block;
  }

  .theme-title {
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--fg, #c8d8f0);
  }

  .form-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10px;
    letter-spacing: 0.08em;
    gap: 12px;
  }

  .form-row label {
    color: var(--fg-dim, #c8d8f088);
    width: 240px;
  }

  .form-row input[type='range'] {
    flex: 1;
    accent-color: var(--accent, #00c8ff);
    cursor: pointer;
  }

  .val-tag {
    font-size: 10px;
    color: var(--accent, #00c8ff);
    width: 48px;
    text-align: right;
  }

  .toggle-switch {
    background: none;
    border: 1px solid var(--accent, #00c8ff44);
    color: var(--fg-dim, #c8d8f077);
    padding: 4px 10px;
    font-family: inherit;
    font-size: 9px;
    letter-spacing: 0.1em;
    border-radius: 3px;
    cursor: pointer;
  }
  .toggle-switch.on {
    color: var(--green, #00ffb3);
    border-color: var(--green, #00ffb366);
    background: rgba(0, 255, 179, 0.08);
  }

  .text-input {
    flex: 1;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--panel-border, #00c8ff33);
    color: var(--fg, #c8d8f0);
    padding: 5px 8px;
    font-family: inherit;
    font-size: 10px;
    border-radius: 3px;
    outline: none;
  }
  .text-input:focus { border-color: var(--accent, #00c8ff); }

  .shortcuts-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
    font-size: 9px;
  }

  .sc-item {
    display: flex;
    justify-content: space-between;
    padding: 4px 6px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  .sc-key { color: var(--accent, #00c8ff); font-weight: bold; }
  .sc-desc { color: var(--fg-dim, #c8d8f066); }

  .save-bar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    padding-top: 6px;
  }

  .save-status {
    font-size: 9px;
    color: var(--green, #00ffb3);
    letter-spacing: 0.1em;
  }

  .save-btn {
    background: var(--accent-glow, #00c8ff18);
    border: 1px solid var(--accent, #00c8ff66);
    color: var(--accent, #00c8ff);
    padding: 8px 16px;
    font-family: inherit;
    font-size: 10px;
    letter-spacing: 0.14em;
    font-weight: bold;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .save-btn:hover {
    background: var(--accent, #00c8ff);
    color: var(--bg, #0a0e1a);
    box-shadow: 0 0 16px var(--accent-glow, #00c8ff66);
  }
</style>
