<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface FsEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size: number;
    modified: string;
  }

  let currentPath = '';
  let entries: FsEntry[] = [];
  let loading = false;
  let error = '';
  let selected = '';

  async function loadDir(path: string) {
    loading = true; error = '';
    try {
      const result = await invoke<FsEntry[]>('fs_list_dir', { path });
      entries = result.sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
      currentPath = path;
    } catch (e) { error = String(e); }
    finally { loading = false; }
  }

  async function goUp() {
    const parts = currentPath.replace(/\\/g, '/').split('/').filter(Boolean);
    if (parts.length <= 1) return;
    parts.pop();
    await loadDir(parts.join('/') || '/');
  }

  function formatSize(bytes: number, isDir: boolean): string {
    if (isDir) return '—';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  onMount(async () => {
    try {
      const home = await invoke<string>('fs_home_dir');
      await loadDir(home);
    } catch { await loadDir('C:/'); }
  });
</script>

<div class="fm">
  <div class="fm-header">◫ FILE MANAGER</div>
  <div class="fm-path">
    <button class="up-btn" on:click={goUp}>↑</button>
    <span class="path-text">{currentPath}</span>
  </div>
  {#if loading}
    <div class="fm-msg">LOADING...</div>
  {:else if error}
    <div class="fm-msg error">{error}</div>
  {:else}
    <div class="fm-list">
      {#each entries as entry}
        <button
          class="fm-entry"
          class:dir={entry.is_dir}
          class:selected={selected === entry.path}
          on:click={() => { selected = entry.path; if (entry.is_dir) loadDir(entry.path); }}
        >
          <span class="icon">{entry.is_dir ? '▶' : '·'}</span>
          <span class="name">{entry.name}</span>
          <span class="size">{formatSize(entry.size, entry.is_dir)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .fm { display:flex; flex-direction:column; height:100%; background:var(--panel-bg,#0d1220ee); border:1px solid var(--panel-border,#00c8ff33); border-radius:6px; font-family:'JetBrains Mono',monospace; overflow:hidden; }
  .fm-header { padding:8px 12px; border-bottom:1px solid var(--panel-border,#00c8ff22); font-size:10px; letter-spacing:0.15em; color:var(--accent,#00c8ff); flex-shrink:0; }
  .fm-path { display:flex; align-items:center; gap:8px; padding:5px 10px; border-bottom:1px solid var(--panel-border,#00c8ff11); flex-shrink:0; }
  .up-btn { background:none; border:1px solid var(--panel-border,#00c8ff22); color:var(--accent,#00c8ff); font-size:12px; cursor:pointer; padding:1px 7px; border-radius:3px; transition:all 0.15s; }
  .up-btn:hover { box-shadow:0 0 6px var(--accent-glow,#00c8ff22); }
  .path-text { font-size:9px; color:var(--fg-dim,#c8d8f044); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .fm-list { flex:1; overflow-y:auto; padding:4px 0; }
  .fm-list::-webkit-scrollbar { width:4px; }
  .fm-list::-webkit-scrollbar-thumb { background:var(--panel-border,#00c8ff22); border-radius:2px; }
  .fm-entry { display:flex; align-items:center; gap:8px; width:100%; background:none; border:none; padding:4px 12px; cursor:pointer; transition:background 0.1s; font-family:inherit; }
  .fm-entry:hover { background:#ffffff06; }
  .fm-entry.selected { background:var(--accent-glow,#00c8ff11); }
  .icon { font-size:9px; color:var(--fg-dim,#c8d8f033); width:10px; }
  .fm-entry.dir .icon { color:var(--accent,#00c8ff); }
  .name { flex:1; font-size:11px; color:var(--fg,#c8d8f0); text-align:left; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .fm-entry.dir .name { color:var(--accent,#00c8ff); }
  .size { font-size:9px; color:var(--fg-dim,#c8d8f033); flex-shrink:0; }
  .fm-msg { padding:20px; font-size:10px; text-align:center; letter-spacing:0.1em; color:var(--fg-dim,#c8d8f044); }
  .fm-msg.error { color:var(--red,#ff4c6a); }
</style>
