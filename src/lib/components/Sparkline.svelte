<script lang="ts">
  export let data: number[] = [];
  export let max: number = 100;
  export let color: string = '#00c8ff';
  export let height: number = 32;
  export let width: number = 160;
  export let filled: boolean = true;

  $: points = (() => {
    if (data.length < 2) return '';
    const step = width / (data.length - 1);
    return data.map((v, i) => {
      const x = i * step;
      const y = height - (Math.min(v, max) / max) * height;
      return `${x},${y}`;
    }).join(' ');
  })();

  $: fillPoints = (() => {
    if (data.length < 2) return '';
    const step = width / (data.length - 1);
    const pts = data.map((v, i) => {
      const x = i * step;
      const y = height - (Math.min(v, max) / max) * height;
      return `${x},${y}`;
    });
    return `0,${height} ${pts.join(' ')} ${width},${height}`;
  })();
</script>

<svg viewBox="0 0 {width} {height}" {width} {height} class="sparkline">
  {#if filled && fillPoints}
    <polygon points={fillPoints} fill={color} fill-opacity="0.12" />
  {/if}
  {#if points}
    <polyline
      points={points}
      fill="none"
      stroke={color}
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  {/if}
  <!-- Current value dot -->
  {#if data.length > 0}
    {@const lastX = width}
    {@const lastY = height - (Math.min(data[data.length - 1], max) / max) * height}
    <circle cx={lastX} cy={lastY} r="2" fill={color} />
  {/if}
</svg>

<style>
  .sparkline { display: block; overflow: visible; }
</style>
